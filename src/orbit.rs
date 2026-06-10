use crate::effects::{
    Comb, CombParams, Compressor, DattorroVerb, Delay, Feedback, ReverbParams, VitalVerb,
};
use crate::types::{ReverbType, StereoFrame, CHANNELS, MAX_BLOCK};

const SILENCE_THRESHOLD: f32 = 1e-7;
const SILENCE_HOLDOFF_SECS: f32 = 1.0;

// SuperDirt-style chain: voices accumulate into `bus`; each FX reads
// `bus * send_level`, adds its wet back into `bus`, in order. Order matters —
// later FX see the running signal including previous FX wet.
//
// Chain order: comb → fb → delay → verb. Tonal/short → spatial/long.
// Reverb last so it captures delay echoes (the load-bearing reason for chaining).
//
// Phase E: `bus` is block-rate. `clear_bus()` zeroes dirty buffers (flag-gated);
// `add_dry(frame, ch, v)` accumulates dry voice output per frame; `process_block(n)`
// runs the FX chain across `bus[..n]` via each FX's Phase-C block API. Sends are
// staged through small stack scratches so the block APIs (which overwrite their
// input) can return wet to be summed onto the bus.
pub struct Orbit {
    pub bus: Box<[StereoFrame; MAX_BLOCK]>,
    /// `bus` may hold nonzero data (written since the last clear). A cleared
    /// flag guarantees the FULL array is zero, so idle orbits skip the clear,
    /// the silence scans, and the final-mix pass entirely.
    pub bus_used: bool,
    /// FX input from `superpan` voices, kept OUT of `bus` so their dry never maps
    /// to the orbit's stereo pair. Merged into `bus` in `process_block` only when
    /// this orbit is room-routed.
    pub fx_send: Box<[StereoFrame; MAX_BLOCK]>,
    /// Wet-only FX result for the room, recovered by snapshot-difference.
    /// Never cleared: the final mix reads it only when `room_active`, and a
    /// room-active block always rewrites `fx_wet[..n]` before returning (the
    /// lone skipping path — the silence bypass — also drops `room_active`).
    pub fx_wet: Box<[StereoFrame; MAX_BLOCK]>,
    /// `fx_send` may hold nonzero data (same lazy-clear contract as `bus_used`).
    pub fx_send_used: bool,
    /// A `superpan` voice fed `fx_send` this chunk (per-chunk, reset in `clear_bus`).
    pub has_fx_send: bool,
    /// A pan voice accumulated dry into `bus` this chunk (per-chunk).
    pub has_pan_dry: bool,
    /// Latched room-routing: set when a superpan voice sends with no pan dry, held
    /// across chunks so the FX tail keeps flowing to the room after the source
    /// stops, released once the orbit goes silent.
    pub room_active: bool,
    pub delay: Delay,
    pub delay_level: f32,
    pub dattorro: [DattorroVerb; CHANNELS],
    pub vital: VitalVerb,
    pub reverb_params: ReverbParams,
    pub verb_level: f32,
    pub comb: [Comb; CHANNELS],
    pub comb_params: CombParams,
    pub comb_level: f32,
    pub fb: Feedback,
    pub fb_level: f32,
    pub comp: Compressor,
    pub comp_orbit: usize,
    pub sr: f32,
    silent_samples: u32,
    silence_holdoff: u32,
}

impl Orbit {
    pub fn new(sr: f32) -> Self {
        let silence_holdoff = (sr * SILENCE_HOLDOFF_SECS) as u32;
        Self {
            bus: Box::new([[0.0; CHANNELS]; MAX_BLOCK]),
            bus_used: false,
            fx_send: Box::new([[0.0; CHANNELS]; MAX_BLOCK]),
            fx_wet: Box::new([[0.0; CHANNELS]; MAX_BLOCK]),
            fx_send_used: false,
            has_fx_send: false,
            has_pan_dry: false,
            room_active: false,
            delay: Delay::new(sr),
            delay_level: 0.0,
            dattorro: std::array::from_fn(|_| DattorroVerb::new(sr)),
            vital: VitalVerb::new(sr),
            reverb_params: ReverbParams::default(),
            verb_level: 0.0,
            comb: [Comb::default(); CHANNELS],
            comb_params: CombParams::default(),
            comb_level: 0.0,
            fb: Feedback::default(),
            fb_level: 0.0,
            comp: Compressor::default(),
            comp_orbit: 0,
            sr,
            silent_samples: silence_holdoff + 1,
            silence_holdoff,
        }
    }

    #[inline]
    pub fn clear_bus(&mut self) {
        // Full-array clears (not `..n`): a clean flag must mean "all zero"
        // even when a later chunk is larger than this one. Only dirty orbits
        // pay them, and they are straight memsets. `fx_wet` is never cleared —
        // see the field invariant.
        if self.bus_used {
            *self.bus = [[0.0; CHANNELS]; MAX_BLOCK];
            self.bus_used = false;
        }
        if self.fx_send_used {
            *self.fx_send = [[0.0; CHANNELS]; MAX_BLOCK];
            self.fx_send_used = false;
        }
        // Per-chunk routing flags. `room_active` latches across chunks and is
        // released in `process_block` on silence, so it is NOT reset here.
        self.has_fx_send = false;
        self.has_pan_dry = false;
    }

    /// True when any orbit FX has a non-zero send level — gate for routing
    /// `superpan` voices into the FX path.
    #[inline]
    pub fn has_any_fx(&self) -> bool {
        self.comb_level > 0.0
            || self.fb_level > 0.0
            || self.delay_level > 0.0
            || self.verb_level > 0.0
    }

    #[inline]
    pub fn add_dry(&mut self, frame: usize, ch: usize, value: f32) {
        self.bus[frame][ch] += value;
        self.bus_used = true;
    }

    /// Block-rate driver for the orbit FX chain. Each enabled FX runs its
    /// Phase-C block API once across `bus[..n]`, staged through a stack
    /// scratch and summed back onto the bus.
    ///
    /// Silence accounting is block-aware (`to_do.md:302`): pre-scan for any
    /// non-zero input; if all zero AND past holdoff, bypass FX and add `n` to
    /// `silent_samples`; otherwise process, then sum-of-abs on `bus[..n]` and
    /// update `silent_samples` accordingly. At `dsp_block_size = 1` (n=1)
    /// this is bit-identical to the legacy per-sample `Orbit::process`.
    pub fn process_block(&mut self, n: usize) {
        debug_assert!(
            n <= MAX_BLOCK,
            "Orbit::process_block: n={n} > MAX_BLOCK={MAX_BLOCK}"
        );
        // Room routing: latch on when a superpan voice sends with no pan dry; the
        // FX run on `bus + fx_send` and only the wet (recovered below) reaches the
        // room. The latch holds the tail to the room after the source stops.
        if self.has_fx_send && !self.has_pan_dry {
            self.room_active = true;
        }

        // FX-less, non-room orbit: the bus is untouched by this function, so
        // the pre-FX input scan and the post-FX energy scan would walk the
        // same data — do the silence accounting with a single pass. Decision-
        // equivalent to the general path below: sum-of-abs over the bus is
        // nonzero iff any sample is nonzero (nonnegative addition; subnormals
        // still sum positive without FTZ, NaN propagates into both compares).
        if !self.has_any_fx() && !self.room_active {
            let mut energy = 0.0_f32;
            if self.bus_used {
                for frame in self.bus.iter().take(n) {
                    energy += frame[0].abs() + frame[1].abs();
                }
            }
            if energy != 0.0 {
                self.silent_samples = 0;
            } else if self.silent_samples > self.silence_holdoff {
                self.silent_samples = self.silent_samples.saturating_add(n as u32);
                return;
            }
            if energy < SILENCE_THRESHOLD * n as f32 {
                self.silent_samples = self.silent_samples.saturating_add(n as u32);
            } else {
                self.silent_samples = 0;
            }
            return;
        }

        let dedicated = self.room_active;
        if dedicated {
            for f in 0..n {
                self.bus[f][0] += self.fx_send[f][0];
                self.bus[f][1] += self.fx_send[f][1];
            }
            // `has_fx_send` ⟺ a superpan voice wrote `fx_send` this chunk, so
            // the merge may have made the bus nonzero without any pan dry.
            if self.has_fx_send {
                self.bus_used = true;
            }
        }

        // Clean flag guarantees an all-zero bus: skip the scan, its result is known.
        let any_input =
            self.bus_used && self.bus.iter().take(n).any(|s| s[0] != 0.0 || s[1] != 0.0);

        if any_input {
            self.silent_samples = 0;
        } else if self.silent_samples > self.silence_holdoff {
            self.silent_samples = self.silent_samples.saturating_add(n as u32);
            self.room_active = false; // tail decayed; release room routing
            return;
        }

        // Past the bypass: the FX chain (or the dedicated merge) writes the bus
        // below, so it must be cleared and mixed from here on.
        self.bus_used = true;

        // Comb (per-channel mono resonator, shared params).
        if self.comb_level > 0.0 {
            let level = self.comb_level;
            let params = self.comb_params;
            let sr = self.sr;
            let mut send = [0.0_f32; MAX_BLOCK];
            for c in 0..CHANNELS {
                for (slot, frame) in send.iter_mut().take(n).zip(self.bus.iter().take(n)) {
                    *slot = frame[c] * level;
                }
                self.comb[c].process_block(&mut send[..n], n, &params, sr);
                for (frame, &wet) in self.bus.iter_mut().take(n).zip(send.iter().take(n)) {
                    frame[c] += wet;
                }
            }
        }

        // Feedback (stereo short delay with cross-channel + LFO).
        if self.fb_level > 0.0 {
            let level = self.fb_level;
            let sr = self.sr;
            let mut send = [[0.0_f32; CHANNELS]; MAX_BLOCK];
            for (slot, frame) in send.iter_mut().take(n).zip(self.bus.iter().take(n)) {
                slot[0] = frame[0] * level;
                slot[1] = frame[1] * level;
            }
            self.fb.process_block(&mut send[..n], n, level, sr);
            for (frame, wet) in self.bus.iter_mut().take(n).zip(send.iter().take(n)) {
                frame[0] += wet[0];
                frame[1] += wet[1];
            }
        }

        // Delay (stereo).
        if self.delay_level > 0.0 {
            let level = self.delay_level;
            let mut send = [[0.0_f32; CHANNELS]; MAX_BLOCK];
            for (slot, frame) in send.iter_mut().take(n).zip(self.bus.iter().take(n)) {
                slot[0] = frame[0] * level;
                slot[1] = frame[1] * level;
            }
            self.delay.process_block(&mut send[..n], n);
            for (frame, wet) in self.bus.iter_mut().take(n).zip(send.iter().take(n)) {
                frame[0] += wet[0];
                frame[1] += wet[1];
            }
        }

        // Reverb — last so it captures delay echoes.
        if self.verb_level > 0.0 {
            let level = self.verb_level;
            let rp = &self.reverb_params;
            match rp.verb_type {
                ReverbType::Plate => {
                    // Each channel into its own Dattorro instance (mono in,
                    // stereo out); the two stereo outputs are summed onto bus.
                    let mut s_l = [[0.0_f32; CHANNELS]; MAX_BLOCK];
                    let mut s_r = [[0.0_f32; CHANNELS]; MAX_BLOCK];
                    for (i, frame) in self.bus.iter().take(n).enumerate() {
                        s_l[i][0] = frame[0] * level;
                        s_r[i][0] = frame[1] * level;
                    }
                    self.dattorro[0].process_block(&mut s_l[..n], n, rp);
                    self.dattorro[1].process_block(&mut s_r[..n], n, rp);
                    for (i, frame) in self.bus.iter_mut().take(n).enumerate() {
                        frame[0] += s_l[i][0] + s_r[i][0];
                        frame[1] += s_l[i][1] + s_r[i][1];
                    }
                }
                ReverbType::Space => {
                    let mut send = [[0.0_f32; CHANNELS]; MAX_BLOCK];
                    for (slot, frame) in send.iter_mut().take(n).zip(self.bus.iter().take(n)) {
                        slot[0] = frame[0] * level;
                        slot[1] = frame[1] * level;
                    }
                    self.vital.process_block(&mut send[..n], n, rp);
                    for (frame, wet) in self.bus.iter_mut().take(n).zip(send.iter().take(n)) {
                        frame[0] += wet[0];
                        frame[1] += wet[1];
                    }
                }
            }
        }

        // Recover wet-only for the room: bus now holds dry+wet, fx_send holds the
        // dry that was merged in, so the difference is the FX return. (When pan
        // dry shares a room-active orbit — misuse — it leaks here; documented.)
        if dedicated {
            for f in 0..n {
                self.fx_wet[f][0] = self.bus[f][0] - self.fx_send[f][0];
                self.fx_wet[f][1] = self.bus[f][1] - self.fx_send[f][1];
            }
        }

        // Post-block silence accounting. Sum-of-abs ≥ threshold·n keeps the
        // counter pinned to 0 while any non-trivial tail energy remains.
        let mut energy = 0.0_f32;
        for frame in self.bus.iter().take(n) {
            energy += frame[0].abs() + frame[1].abs();
        }
        if energy < SILENCE_THRESHOLD * n as f32 {
            self.silent_samples = self.silent_samples.saturating_add(n as u32);
        } else {
            self.silent_samples = 0;
        }
    }
}
