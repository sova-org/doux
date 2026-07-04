//! The realtime hot-swap loop and crossfade.
//!
//! The driver-agnostic [`EngineHost`] owns the running [`Engine`], performs the realtime-safe
//! swap, and mixes a displaced engine out over a crossfade. Split from `engine` so the swap/fade
//! state machine and the canonical sample clock have one home, separate from the executor they drive.

use crate::engine::{Engine, MAX_BLOCK};

/// A displaced engine still sounding through a crossfade: retained by [`EngineHost::swap_to`]
/// for `fade_end - fade_start` frames and mixed out linearly by [`EngineHost::process`].
struct Outgoing {
    engine: Engine,
    /// Clock position where the fade began (gain 1) …
    fade_start: u64,
    /// … and where it ends (gain 0); the engine retires to `finished` once the clock passes it.
    fade_end: u64,
}

/// The driver-agnostic core of the realtime loop: owns the running [`Engine`] and the
/// realtime-safe swap. Every driver — CPAL, offline, the WASM worklet — drives the same
/// host, so state reconciliation happens in one place regardless of who owns the audio
/// loop and how (or whether) programs are queued.
///
/// **The crossfade lives here.** A faded swap retains the displaced engine as [`Outgoing`] and
/// `process` renders *both* engines from the same clock, inputs, and control plane, writing the
/// linear per-frame mix `g·new + (1−g)·old`. Because state migration makes every unchanged cone
/// of the graph sample-identical in the two engines, the linear mix cancels to identity (within
/// ~1 ulp) on everything an edit did not touch and degenerates to a crossfade on exactly what it
/// did — per-slot fades with no slot-awareness in this code. Linear is load-bearing: an
/// equal-power curve would dip every *unchanged* signal by up to 3 dB mid-fade. See
/// `docs/design/2026-06-18-arf-supercollider-roast-and-refactor.md`.
///
/// Caller contract: drain [`take_finished`](Self::take_finished) after every `swap_to` and
/// `process` (native recycles it off the audio thread; single-threaded drivers drop it inline),
/// so at most one retired engine is ever held.
pub struct EngineHost {
    current: Engine,
    /// The engine a faded swap displaced, still rendering until its fade window closes.
    outgoing: Option<Outgoing>,
    /// A retired engine (fade complete, or hard-dropped by a mid-fade swap) awaiting disposal
    /// off the realtime path — drained by [`take_finished`](Self::take_finished).
    finished: Option<Engine>,
    /// Render scratch for the outgoing engine ([`MAX_BLOCK`] f32s), allocated at construction
    /// (control thread) so the dual render never touches the heap.
    fade_scratch: Box<[f32]>,
    /// All-zero input block for an outgoing engine whose input width differs from the
    /// current's (interleaved widths cannot share a block); the fade masks the silence.
    zero_input: Box<[f32]>,
    /// The one canonical sample clock: total frames rendered since the host started, advanced
    /// by every [`process`](Self::process). It lives here, *outside* the swapped [`Engine`], so a
    /// hot-swap never resets it — musical time continues across a live edit by construction, with
    /// no carry rule. Passed down to the engine each block as its `block_start_pos`. A `u64` frame
    /// count is exact for the life of the process (~12 million years at 48 kHz); only the windowed
    /// [`crate::ir::NOW_WINDOW`] view reaches the f32 DSP core. The fade gain derives from it,
    /// so the mix is chunk-invariant by construction.
    frames_elapsed: u64,
}

impl EngineHost {
    /// Start the host running `initial` (typically [`Program::silent`]) with the clock at zero.
    pub fn new(initial: Engine) -> Self {
        EngineHost {
            current: initial,
            outgoing: None,
            finished: None,
            fade_scratch: vec![0.0; MAX_BLOCK].into_boxed_slice(),
            zero_input: vec![0.0; MAX_BLOCK].into_boxed_slice(),
            frames_elapsed: 0,
        }
    }

    /// Adopt `next`, carrying state forward from the engine it displaces (per `next`'s
    /// migration plan, see [`Engine::adopt_state_from`]), and install it as current.
    ///
    /// With `fade_frames == 0` this is today's instant swap: buffers donate when flagged and
    /// the displaced engine is **returned** (`Some`) for disposal off the realtime path. With
    /// `fade_frames > 0` the displaced engine is **retained** as [`Outgoing`] (`None` is
    /// returned) and buffer regions are *copied* per `next`'s plan — donation is impossible
    /// while the outgoing engine keeps using its arena.
    ///
    /// Two engines max: a swap arriving mid-fade hard-drops the old outgoing into `finished`
    /// (it was at gain `1−g` and falling, so the click risk shrinks as the fade progresses).
    /// The caller must have drained [`take_finished`](Self::take_finished) before swapping.
    /// Realtime-safe: bounded slice copies, no allocation, no rings.
    pub fn swap_to(&mut self, mut next: Engine) -> Option<Engine> {
        debug_assert!(self.finished.is_none(), "drain take_finished() before swapping");
        next.adopt_state_from(&self.current);
        if next.fade_frames() == 0 {
            // Carry delay-line contents when the buffer layouts matched (decided on the control
            // thread): an O(1) arena swap. The length guard mirrors the migration bounds check —
            // a plan built against the wrong predecessor degrades to a fresh-zero arena rather
            // than swapping mismatched sizes.
            if next.donate_buffers() && next.buffers().len() == self.current.buffers().len() {
                next.donate_buffers_from(&mut self.current);
            }
            // An instant swap replaces the whole mix, so a fade in flight retires with it.
            if let Some(out) = self.outgoing.take() {
                self.finished = Some(out.engine);
            }
            Some(std::mem::replace(&mut self.current, next))
        } else {
            next.adopt_buffers_from(&self.current);
            if let Some(out) = self.outgoing.take() {
                self.finished = Some(out.engine);
            }
            let fade_frames = next.fade_frames();
            let displaced = std::mem::replace(&mut self.current, next);
            self.outgoing = Some(Outgoing {
                engine: displaced,
                fade_start: self.frames_elapsed,
                fade_end: self.frames_elapsed + fade_frames,
            });
            None
        }
    }

    /// Take a retired engine (its fade completed, or it was hard-dropped by a newer swap) for
    /// disposal off the realtime path. Native pushes it to the recycle ring; the WASM worklet
    /// and offline drivers drop it inline.
    pub fn take_finished(&mut self) -> Option<Engine> {
        self.finished.take()
    }

    /// Fill `out` with the next frames from the current engine, reading `input` and the
    /// per-block `control` plane (see [`Engine::process`]). Feeds the engine the clock position
    /// of the first frame, then advances the canonical clock by the frames rendered — so `now`
    /// is continuous across every block and chunk, and across hot-swaps.
    ///
    /// While a fade is in flight, the outgoing engine renders the same frames (same clock, same
    /// control; the same input when the widths match, silence otherwise) into the preallocated
    /// scratch, and each frame is mixed `g·current + (1−g)·outgoing` with
    /// `g = (pos − fade_start) / fade_len` computed from the absolute clock — plain f32 ops
    /// (Rust never auto-fuses into FMA), so the mix is deterministic and chunk-invariant.
    pub fn process(&mut self, input: &[f32], control: &[f32], out: &mut [f32]) {
        let cur_ch = self.current.channels().max(1);
        let frames = out.len() / cur_ch;
        match &mut self.outgoing {
            None => self.current.process_from(self.frames_elapsed, input, control, out),
            Some(og) => {
                let cur_in = self.current.in_channels();
                let old_ch = og.engine.channels().max(1);
                let old_in = og.engine.in_channels();
                // Mix over the channels both engines have; wider current channels keep their
                // rendered value (fill-from-current), wider outgoing channels go unheard.
                let mix_ch = cur_ch.min(old_ch);
                let share_input = old_in == cur_in;
                // Chunk so the outgoing render (and its zero input) fit the scratches whatever
                // the caller's block size — the WASM worklet calls with un-chunked blocks.
                let fpc = (MAX_BLOCK / old_ch.max(old_in).max(1)).max(1);
                let fade_len = (og.fade_end - og.fade_start) as f32;
                let mut done = 0usize;
                while done < frames {
                    let f = (frames - done).min(fpc);
                    let pos = self.frames_elapsed + done as u64;
                    let out_chunk = &mut out[done * cur_ch..(done + f) * cur_ch];
                    let in_chunk = &input[done * cur_in..(done + f) * cur_in];
                    self.current.process_from(pos, in_chunk, control, out_chunk);
                    let old_input: &[f32] =
                        if share_input { in_chunk } else { &self.zero_input[..f * old_in] };
                    let scratch = &mut self.fade_scratch[..f * old_ch];
                    og.engine.process_from(pos, old_input, control, scratch);
                    for i in 0..f {
                        let p = pos + i as u64;
                        if p >= og.fade_end {
                            continue; // gain 1: `out` already holds the current engine
                        }
                        let g = (p - og.fade_start) as f32 / fade_len;
                        for c in 0..mix_ch {
                            let o = i * cur_ch + c;
                            out_chunk[o] = g * out_chunk[o] + (1.0 - g) * scratch[i * old_ch + c];
                        }
                    }
                    done += f;
                }
            }
        }
        self.frames_elapsed += frames as u64;
        // Retire a completed fade (deferred if a hard-dropped engine is still awaiting drain —
        // past `fade_end` the outgoing contributes nothing, it only waits).
        if self
            .outgoing
            .as_ref()
            .is_some_and(|og| og.fade_end <= self.frames_elapsed)
            && self.finished.is_none()
        {
            self.finished = self.outgoing.take().map(|og| og.engine);
        }
    }

    /// The current engine's output channel count (≥ 1) — audio channels plus taps.
    pub fn channels(&self) -> usize {
        self.current.channels()
    }

    /// The current engine's device-facing audio channel count (the rest of `channels` are taps).
    pub fn audio_channels(&self) -> usize {
        self.current.audio_channels()
    }

    /// The current engine's observation-tap names, aligned with the trailing outputs.
    pub fn tap_names(&self) -> &[String] {
        self.current.tap_names()
    }

    /// The current engine's audio input channel count (the width of the input block it
    /// expects per frame).
    pub fn in_channels(&self) -> usize {
        self.current.in_channels()
    }

    /// The control-plane width the host must supply: the current engine's — or, while a fade
    /// is in flight, the wider of the two engines', since both render from the one slice.
    pub fn control_len(&self) -> usize {
        let cur = self.current.control_len();
        match &self.outgoing {
            Some(og) => cur.max(og.engine.control_len()),
            None => cur,
        }
    }

    /// The current engine's voice-pool size (the host spreads MIDI notes across this many).
    pub fn voice_count(&self) -> usize {
        self.current.voice_count()
    }

    /// The canonical sample clock: total frames rendered since the host started. A scheduler
    /// compares event times against this to decide which events are due.
    pub fn frames_elapsed(&self) -> u64 {
        self.frames_elapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{skipped_migrations, Carryover};
    use crate::compile::compile;
    use crate::graph::Graph;
    use crate::testutil::{dc, graph_of, osc, u};

    fn engine(graph: &Graph) -> Engine {
        Engine::new(compile(graph, 48_000.0))
    }

    /// `1 0.05 delay out` — a constant into a 50 ms delay line (the buffer-carry fixture).
    fn delay_line() -> Graph {
        graph_of(|g| {
            let s = g.constant(1.0);
            let t = g.constant(0.05);
            vec![g.ugen(u("delay"), vec![s, t])]
        })
    }

    /// `freq <name> secs delay out` — an oscillator into a delay line.
    fn osc_delay(name: &str, freq: f32, secs: f32) -> Graph {
        graph_of(|g| {
            let f = g.constant(freq);
            let o = g.ugen(u(name), vec![f]);
            let t = g.constant(secs);
            vec![g.ugen(u("delay"), vec![o, t])]
        })
    }

    /// `1 record loop drop play loop out` — a one-second looper: `record` sinks the constant
    /// into a named buffer, `play` reads it back.
    fn looper() -> Graph {
        graph_of(|g| {
            let buf = g.new_buffer(1.0);
            let one = g.constant(1.0);
            let rec = g.ugen_buf(u("record"), vec![one], buf);
            g.add_sink(rec); // the write must run even though `drop` discards the passthrough
            vec![g.ugen_buf(u("play"), vec![], buf)]
        })
    }

    /// `buf big 12  noise record big drop  play big  0.01 delay out` — a 12 s named buffer
    /// (oversized for the fade copy cap) feeding a small delay.
    fn big_looper() -> Graph {
        graph_of(|g| {
            let big = g.new_buffer(12.0);
            let n = g.ugen(u("noise"), vec![]);
            let rec = g.ugen_buf(u("record"), vec![n], big);
            g.add_sink(rec);
            let pl = g.ugen_buf(u("play"), vec![], big);
            let t = g.constant(0.01);
            vec![g.ugen(u("delay"), vec![pl, t])]
        })
    }

    /// `[ a b ] out` — a stereo pair of DC constants.
    fn stereo(a: f32, b: f32) -> Graph {
        graph_of(|g| vec![g.constant(a), g.constant(b)])
    }

    #[test]
    fn a_fade_zero_swap_returns_the_displaced_engine() {
        let mut host = EngineHost::new(engine(&dc(0.1)));
        let displaced = host.swap_to(engine(&dc(0.2))).expect("an instant swap returns it");

        // The host now runs the new engine.
        let mut frame = [0.0; 1];
        host.process(&[], &[], &mut frame);
        assert_eq!(frame[0], 0.2);

        // The displaced engine is returned intact for the caller to recycle.
        let mut displaced = displaced;
        let mut old = [0.0; 1];
        displaced.process(&[], &[], &mut old);
        assert_eq!(old[0], 0.1);
    }

    /// The expected linear mix for one frame of a fade — the *identical* arithmetic
    /// [`EngineHost::process`] performs (plain f32 ops, no FMA), so tests assert exact equality.
    fn mixed(p: u64, fade_start: u64, fade_end: u64, cur: f32, old: f32) -> f32 {
        if p >= fade_end {
            return cur;
        }
        let g = (p - fade_start) as f32 / ((fade_end - fade_start) as f32);
        g * cur + (1.0 - g) * old
    }

    #[test]
    fn a_faded_swap_retains_the_outgoing_and_mixes_linearly() {
        let mut host = EngineHost::new(engine(&dc(1.0)));
        host.process(&[], &[], &mut [0.0; 16]); // clock at 16: the fade starts off zero
        assert!(host.swap_to(engine(&dc(0.0)).with_fade_frames(8)).is_none(), "retained");

        let mut out = [0.0_f32; 16];
        host.process(&[], &[], &mut out);
        for (i, &got) in out.iter().enumerate() {
            let want = mixed(16 + i as u64, 16, 24, 0.0, 1.0);
            assert_eq!(got, want, "frame {i}: linear ramp from the old 1.0 to the new 0.0");
        }
    }

    #[test]
    fn the_finished_engine_surfaces_after_the_fade() {
        let mut host = EngineHost::new(engine(&dc(1.0)));
        assert!(host.swap_to(engine(&dc(0.0)).with_fade_frames(8)).is_none());
        assert!(host.take_finished().is_none(), "the fade is still in flight");

        host.process(&[], &[], &mut [0.0; 16]); // render past fade_end = 8
        let mut finished = host.take_finished().expect("the outgoing engine retires");
        let mut frame = [0.0; 1];
        finished.process(&[], &[], &mut frame);
        assert_eq!(frame[0], 1.0, "the retired engine is the displaced one, intact");
        assert!(host.take_finished().is_none(), "drained once");
    }

    #[test]
    fn a_second_swap_mid_fade_hard_drops_the_outgoing() {
        let mut host = EngineHost::new(engine(&dc(1.0)));
        assert!(host.swap_to(engine(&dc(0.5)).with_fade_frames(100)).is_none());
        host.process(&[], &[], &mut [0.0; 8]); // mid-fade

        // A third generation arrives: the old outgoing (1.0) hard-drops; 0.5 becomes outgoing.
        assert!(host.swap_to(engine(&dc(0.0)).with_fade_frames(100)).is_none());
        let mut dropped = host.take_finished().expect("the first outgoing was hard-dropped");
        let mut frame = [0.0; 1];
        dropped.process(&[], &[], &mut frame);
        assert_eq!(frame[0], 1.0);

        // The new fade mixes generations 2 and 3 over a fresh full-length window (start = 8).
        let mut out = [0.0_f32; 4];
        host.process(&[], &[], &mut out);
        for (i, &got) in out.iter().enumerate() {
            assert_eq!(got, mixed(8 + i as u64, 8, 108, 0.0, 0.5), "frame {i}");
        }
    }

    #[test]
    fn a_fade_zero_swap_mid_fade_also_retires_the_outgoing() {
        let mut host = EngineHost::new(engine(&dc(1.0)));
        assert!(host.swap_to(engine(&dc(0.5)).with_fade_frames(100)).is_none());
        host.process(&[], &[], &mut [0.0; 8]);

        // An instant swap replaces the whole mix: it returns the displaced current AND
        // retires the in-flight outgoing.
        let displaced = host.swap_to(engine(&dc(0.25))).expect("instant swap returns");
        drop(displaced);
        assert!(host.take_finished().is_some(), "the in-flight fade retired with it");
        let mut frame = [0.0; 1];
        host.process(&[], &[], &mut frame);
        assert_eq!(frame[0], 0.25, "no mix remains");
    }

    #[test]
    fn fade_mix_is_chunk_invariant() {
        // The gain derives from the absolute clock, so one 4096-frame render and 32×128-frame
        // renders of the same schedule are bit-identical — the worklet (128) and the native
        // shell (device-sized) must sound the same fade.
        let run = |chunk: usize| -> Vec<f32> {
            let mut host = EngineHost::new(engine(&osc("sine", 440.0)));
            host.process(&[], &[], &mut [0.0; 256]);
            assert!(host.swap_to(engine(&osc("saw", 220.0)).with_fade_frames(1000)).is_none());
            let mut out = vec![0.0_f32; 4096];
            for c in out.chunks_mut(chunk) {
                host.process(&[], &[], c);
            }
            out
        };
        assert_eq!(run(4096), run(128));
    }

    #[test]
    fn channel_width_mismatch_mixes_over_min_and_fills_from_current() {
        // Mono 1.0 fading into a stereo program: channel 0 mixes, channel 1 is pure current.
        let mut host = EngineHost::new(engine(&dc(1.0)));
        assert!(host.swap_to(engine(&stereo(0.0, 0.25)).with_fade_frames(8)).is_none());
        let mut out = [0.0_f32; 8]; // 4 stereo frames
        host.process(&[], &[], &mut out);
        for i in 0..4 {
            assert_eq!(out[i * 2], mixed(i as u64, 0, 8, 0.0, 1.0), "ch0 mixes at frame {i}");
            assert_eq!(out[i * 2 + 1], 0.25, "ch1 is filled from the current engine");
        }
    }

    #[test]
    fn buffer_copies_apply_on_adoption_under_guards() {
        use crate::reconcile::{BufCopy, buffer_copy_plan, buffer_regions};
        let sr = 48_000.0;
        let prog = |graph: &Graph| compile(graph, sr);

        // Fill a delay line, then fade-swap to the same patch with the copy plan: the line
        // carries (donation is impossible mid-fade).
        let p1 = prog(&delay_line());
        let p2 = prog(&delay_line());
        let copies = buffer_copy_plan(&buffer_regions(&p1), &buffer_regions(&p2)).copies;
        let mut host = EngineHost::new(Engine::new(p1));
        host.process(&[], &[], &mut [0.0; 1000]);
        let filled = host.current.buffers().to_vec();
        assert!(filled.iter().any(|&s| s != 0.0));
        assert!(
            host.swap_to(Engine::new(p2).with_buffer_copies(copies).with_fade_frames(8)).is_none()
        );
        assert_eq!(host.current.buffers(), filled.as_slice(), "the line copied across");

        // An out-of-range copy is skipped (and counted), never a panic on the audio thread.
        let before = skipped_migrations();
        let bad = vec![BufCopy { old_base: u32::MAX, new_base: 0, len: 16 }];
        host.process(&[], &[], &mut [0.0; 100]); // retire the fade
        host.take_finished();
        host.swap_to(Engine::new(prog(&delay_line())).with_buffer_copies(bad).with_fade_frames(8));
        assert_eq!(skipped_migrations(), before + 1, "the bad copy was counted, not applied");
    }

    #[test]
    fn swap_to_carries_state_when_the_new_engine_has_a_migration_plan() {
        use crate::reconcile::{plan, signatures};
        let sr = 48_000.0;
        let prog_a = compile(&osc("sine", 440.0), sr);
        let prog_b = compile(&osc("sine", 440.0), sr);
        let migration = plan(&signatures(&prog_a), &signatures(&prog_b));

        // Advance A's phase by rendering a block, then snapshot it.
        let mut a = Engine::new(prog_a);
        a.process(&[], &[], &mut [0.0; 64]);
        let carried = a.state().to_vec();
        assert_ne!(carried[0], 0.0, "phase should have advanced");

        // Swapping to B (which carries A's migration plan) installs A's phase, not zero.
        let b = Engine::new(prog_b).with_migration(migration);
        let mut host = EngineHost::new(a);
        host.swap_to(b);
        assert_eq!(host.current.state(), carried.as_slice());
    }

    #[test]
    fn swapping_a_buffer_ugen_stays_finite() {
        // A delay declares a sample-memory buffer. Across a hot-swap the line resets while
        // the masked ring keeps every index in range — rendering must stay finite and
        // never panic.
        let mut host = EngineHost::new(engine(&osc_delay("sine", 440.0, 0.01)));
        let mut frame = [0.0; 1];
        for _ in 0..64 {
            host.process(&[], &[], &mut frame);
        }
        host.swap_to(engine(&osc_delay("sine", 220.0, 0.02)));
        for _ in 0..64 {
            host.process(&[], &[], &mut frame);
            assert!(frame[0].is_finite(), "delay output went non-finite after a swap");
        }
    }

    #[test]
    fn swap_to_donates_buffers_when_the_flag_is_set() {
        // A constant 1 into a delay fills its line; a same-layout edit with the donation flag
        // set carries those contents across the swap (an O(1) arena swap), not a reset.
        let mut host = EngineHost::new(engine(&delay_line()));
        host.process(&[], &[], &mut vec![0.0_f32; 1000]);
        let filled = host.current.buffers().to_vec();
        assert!(filled.iter().any(|&s| s != 0.0), "delay line should hold signal");

        // The control thread sets the flag when the layout matches; here it does.
        let b = engine(&delay_line()).with_buffer_donation(true);
        host.swap_to(b);
        assert_eq!(host.current.buffers(), filled.as_slice(), "delay contents donated");
    }

    #[test]
    fn swap_to_keeps_fresh_buffers_without_the_donation_flag() {
        // Same patch, but the flag is left unset (a layout change, say): the new line starts
        // fresh-zero instead of inheriting the old contents.
        let mut host = EngineHost::new(engine(&delay_line()));
        host.process(&[], &[], &mut vec![0.0_f32; 1000]);
        assert!(host.current.buffers().iter().any(|&s| s != 0.0));

        host.swap_to(engine(&delay_line())); // no with_buffer_donation → false
        assert!(
            host.current.buffers().iter().all(|&s| s == 0.0),
            "without donation the delay line resets"
        );
    }

    #[test]
    fn swap_to_donates_a_named_buffer() {
        // A `record` fills a named buffer; a same-layout re-eval with the donation flag carries
        // those contents across the swap, so a loop survives an edit instead of resetting.
        let mut host = EngineHost::new(engine(&looper()));
        host.process(&[], &[], &mut vec![0.0_f32; 1000]);
        let filled = host.current.buffers().to_vec();
        assert!(filled.contains(&1.0), "named buffer should hold the recorded signal");

        let b = engine(&looper()).with_buffer_donation(true);
        host.swap_to(b);
        assert_eq!(host.current.buffers(), filled.as_slice(), "named buffer contents donated");
    }

    /// Stage `src` from `carry` and commit, returning a host running it — the frontends'
    /// stage/commit sequence the `carryover_*` tests below exercise.
    fn staged_host(carry: &mut Carryover, graph: &Graph) -> EngineHost {
        let staged = carry.stage(engine(graph), 0.0);
        *carry = staged.next;
        EngineHost::new(staged.engine)
    }

    #[test]
    fn carryover_stages_donation_on_an_instant_layout_match() {
        let mut carry = Carryover::default();
        let mut host = staged_host(&mut carry, &delay_line());
        host.process(&[], &[], &mut vec![0.0_f32; 1000]);
        let filled = host.current.buffers().to_vec();
        assert!(filled.iter().any(|&s| s != 0.0), "the delay line holds signal");

        let staged = carry.stage(engine(&delay_line()), 0.0);
        assert_eq!(staged.declined, 0, "an instant swap never declines");
        host.swap_to(staged.engine);
        assert_eq!(host.current.buffers(), filled.as_slice(), "the line donated across");
    }

    #[test]
    fn carryover_copies_buffers_through_a_fade() {
        let mut carry = Carryover::default();
        let mut host = staged_host(&mut carry, &delay_line());
        host.process(&[], &[], &mut vec![0.0_f32; 1000]);
        let filled = host.current.buffers().to_vec();

        let staged = carry.stage(engine(&delay_line()), 8.0 / 48_000.0);
        assert!(host.swap_to(staged.engine).is_none(), "a faded swap retains the outgoing");
        assert_eq!(host.current.buffers(), filled.as_slice(), "the line copied across");
    }

    #[test]
    fn carryover_surfaces_fade_copy_declines() {
        // 12 s @ 48 kHz exceeds the per-buffer copy cap (see `reconcile::buffer_copy_plan`):
        // a faded re-send must surface the decline for the frontends to warn on.
        let src = big_looper();
        let mut carry = Carryover::default();
        staged_host(&mut carry, &src);
        let staged = carry.stage(engine(&src), 0.5);
        assert_eq!(staged.declined, 1, "the oversized buffer's decline surfaces");
    }

    #[test]
    fn an_uncommitted_stage_plans_against_the_last_committed_engine() {
        let mut carry = Carryover::default();
        let mut host = staged_host(&mut carry, &osc("sine", 440.0));
        host.process(&[], &[], &mut [0.0; 64]);
        let phase = host.current.state()[0];
        assert_ne!(phase, 0.0, "the phase advanced");

        // A dropped edit (native: the ring send failed) — staged but never committed.
        drop(carry.stage(engine(&dc(0.1)), 0.0));

        // The next stage must plan against the committed sine, not the dropped constant:
        // if `stage` eagerly committed, this plan would be empty and the phase would reset.
        let staged = carry.stage(engine(&osc("sine", 440.0)), 0.0);
        host.swap_to(staged.engine);
        assert_eq!(host.current.state()[0], phase, "the phase carried across");
    }
}
