//! Realtime-safe MIDI voice allocation.
//!
//! Maps incoming MIDI notes onto a fixed pool of voice slots in the control plane, on the
//! audio-callback (control) side, so the replicated voices (see [`crate::graph`]) read their
//! gate/notefreq/vel from the slot the allocator wrote. The pool is sized once at construction
//! to [`crate::graph::MAX_VOICES`]; an `active` count (the running program's `voice_count`)
//! bounds how many are in use. Allocation-free after construction — the audio thread only
//! mutates the fixed `slots` vec and the host's control block.

use crate::graph::{GATE_LANE, LANES_PER_VOICE, MAX_VOICES, NOTEFREQ_LANE, VEL_LANE};

/// One voice slot's bookkeeping; the audio values (gate/notefreq/vel) live in the control plane.
#[derive(Clone, Copy)]
struct Slot {
    /// The MIDI note currently sounding on this voice, or `None` when free.
    note: Option<u8>,
    /// A monotonic stamp of when this voice was last triggered, for oldest-first stealing.
    started: u64,
}

/// A fixed pool of voices, assigning MIDI notes to control-plane slots. The first `active`
/// slots are in use (the running program's `voice_count`); the rest lie idle until a poly
/// edit raises the count.
pub struct VoiceAlloc {
    slots: Vec<Slot>,
    /// Voices currently in use (1..=`MAX_VOICES`). Notes are assigned within `slots[..active]`.
    active: usize,
    /// Next trigger stamp (monotonic), for oldest-first note-stealing.
    age: u64,
}

impl Default for VoiceAlloc {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceAlloc {
    /// A pool of [`MAX_VOICES`] slots, monophonic until [`VoiceAlloc::set_active`] raises it.
    /// Allocates its slot bookkeeping once, here (never on the audio thread).
    pub fn new() -> Self {
        VoiceAlloc { slots: vec![Slot { note: None, started: 0 }; MAX_VOICES], active: 1, age: 0 }
    }

    /// Set how many voices are in use (the running program's `voice_count`), clamped to
    /// `1..=MAX_VOICES`. Cheap; called per block as the engine hot-swaps. When the pool shrinks,
    /// release any voice that falls outside it: drop its gate in `control` and free the slot, so
    /// a note held on a now-inactive voice cannot stay stuck (and resurrect if the pool grows
    /// back over it). Realtime-safe: a bounded scan, no allocation.
    pub fn set_active(&mut self, n: usize, control: &mut [f32]) {
        let n = n.clamp(1, MAX_VOICES);
        for v in n..self.active {
            if self.slots[v].note.is_some() {
                self.slots[v].note = None;
                control[v * LANES_PER_VOICE + GATE_LANE] = 0.0;
            }
        }
        self.active = n;
    }

    /// Handle a note-on: pick the voice already holding `note` (retrigger), else a free voice,
    /// else steal the oldest; write its gate/notefreq/vel into `control`. Realtime-safe.
    pub fn note_on(&mut self, note: u8, vel: u8, control: &mut [f32]) {
        let v = self.pick_voice(note);
        self.slots[v].note = Some(note);
        self.slots[v].started = self.age;
        self.age += 1;
        let base = v * LANES_PER_VOICE;
        control[base + GATE_LANE] = 1.0;
        control[base + NOTEFREQ_LANE] = midi_to_hz(note);
        control[base + VEL_LANE] = vel as f32 / 127.0;
    }

    /// Handle a note-off: drop the gate of the voice holding `note`, freeing the slot. notefreq
    /// and vel are left as-is so the envelope's release tail decays at pitch. A note that is not
    /// currently held is ignored (e.g. an off after a steal).
    pub fn note_off(&mut self, note: u8, control: &mut [f32]) {
        if let Some(v) = self.held(note) {
            self.slots[v].note = None;
            control[v * LANES_PER_VOICE + GATE_LANE] = 0.0;
        }
    }

    /// The active slot holding `note`, if any.
    fn held(&self, note: u8) -> Option<usize> {
        self.slots[..self.active].iter().position(|s| s.note == Some(note))
    }

    /// Pick a voice for `note`: reuse the slot already holding it, else the first free slot,
    /// else steal the oldest-triggered voice. Always within `slots[..active]`.
    fn pick_voice(&self, note: u8) -> usize {
        if let Some(v) = self.held(note) {
            return v;
        }
        if let Some(v) = self.slots[..self.active].iter().position(|s| s.note.is_none()) {
            return v;
        }
        self.slots[..self.active]
            .iter()
            .enumerate()
            .min_by_key(|(_, s)| s.started)
            .map(|(v, _)| v)
            .unwrap_or(0)
    }
}

/// MIDI note number → frequency in Hz (A4 = note 69 = 440 Hz, equal temperament). Runs on the
/// control side (the allocator), never inside a UGen `tick`, so its `powf` is off the JIT's
/// bit-exact path.
pub fn midi_to_hz(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CONTROL_WIDTH, GATE_LANE, LANES_PER_VOICE, NOTEFREQ_LANE, VEL_LANE};

    fn gate(control: &[f32], v: usize) -> f32 {
        control[v * LANES_PER_VOICE + GATE_LANE]
    }

    #[test]
    fn note_on_writes_the_voice_lanes() {
        let mut alloc = VoiceAlloc::new();
        let mut control = vec![0.0_f32; CONTROL_WIDTH];
        alloc.set_active(4, &mut control);
        alloc.note_on(69, 127, &mut control); // A4 at full velocity → voice 0
        assert_eq!(gate(&control, 0), 1.0);
        assert!((control[NOTEFREQ_LANE] - 440.0).abs() < 1e-3);
        assert!((control[VEL_LANE] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn note_off_clears_the_gate() {
        let mut alloc = VoiceAlloc::new();
        let mut control = vec![0.0_f32; CONTROL_WIDTH];
        alloc.set_active(4, &mut control);
        alloc.note_on(60, 100, &mut control);
        alloc.note_off(60, &mut control);
        assert_eq!(gate(&control, 0), 0.0, "gate drops on note-off");
        // notefreq is left so the release tail decays at pitch.
        assert!(control[NOTEFREQ_LANE] > 0.0);
    }

    #[test]
    fn distinct_notes_spread_across_voices() {
        let mut alloc = VoiceAlloc::new();
        let mut control = vec![0.0_f32; CONTROL_WIDTH];
        alloc.set_active(4, &mut control);
        alloc.note_on(60, 100, &mut control);
        alloc.note_on(64, 100, &mut control);
        alloc.note_on(67, 100, &mut control);
        assert_eq!(gate(&control, 0), 1.0);
        assert_eq!(gate(&control, 1), 1.0);
        assert_eq!(gate(&control, 2), 1.0);
        assert_eq!(gate(&control, 3), 0.0, "the fourth voice stays free");
    }

    #[test]
    fn a_fifth_note_steals_the_oldest_voice() {
        let mut alloc = VoiceAlloc::new();
        let mut control = vec![0.0_f32; CONTROL_WIDTH];
        alloc.set_active(2, &mut control); // only two voices
        alloc.note_on(60, 100, &mut control); // voice 0 (oldest)
        alloc.note_on(64, 100, &mut control); // voice 1
        alloc.note_on(67, 100, &mut control); // steals voice 0
        // Voice 0 now plays note 67's pitch (the oldest was stolen).
        assert!((control[NOTEFREQ_LANE] - midi_to_hz(67)).abs() < 1e-3);
        assert_eq!(gate(&control, 0), 1.0);
        assert_eq!(gate(&control, 1), 1.0);
    }

    #[test]
    fn repeated_note_reuses_its_voice() {
        let mut alloc = VoiceAlloc::new();
        let mut control = vec![0.0_f32; CONTROL_WIDTH];
        alloc.set_active(4, &mut control);
        alloc.note_on(60, 100, &mut control);
        alloc.note_on(60, 110, &mut control); // same note again → same voice, not a new one
        assert_eq!(gate(&control, 1), 0.0, "no second voice was used");
        assert!((control[VEL_LANE] - 110.0 / 127.0).abs() < 1e-6, "velocity updated");
    }

    #[test]
    fn middle_a_is_440() {
        assert!((midi_to_hz(69) - 440.0).abs() < 1e-3);
        assert!((midi_to_hz(81) - 880.0).abs() < 1e-2, "an octave up doubles");
    }

    #[test]
    fn shrinking_the_pool_releases_voices_outside_it() {
        // Play a chord on 8 voices, then edit the patch down to 2 voices. A note held on a
        // now-inactive voice must be released — not left with its gate stuck at 1.0, which used
        // to resurrect as a drone when the pool later grew back over it.
        let mut alloc = VoiceAlloc::new();
        let mut control = vec![0.0_f32; CONTROL_WIDTH];
        alloc.set_active(8, &mut control);
        for note in 60..68 {
            alloc.note_on(note, 100, &mut control); // fills voices 0..8
        }
        assert_eq!(gate(&control, 5), 1.0, "voice 5 is held before the shrink");

        alloc.set_active(2, &mut control); // hot-swap to a 2-voice patch
        assert_eq!(gate(&control, 5), 0.0, "a voice outside the shrunk pool is released");

        alloc.set_active(8, &mut control); // grow back: the released voice must stay silent
        assert_eq!(gate(&control, 5), 0.0, "no resurrected drone on regrow");
    }
}
