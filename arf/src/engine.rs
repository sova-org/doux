//! An [`Engine`] bundles a [`Program`] with the executor state that runs it.
//!
//! This is the unit handed to the audio thread. The control thread builds a fully
//! allocated `Engine` (program + sized executor state) so that adopting a new one on
//! the audio thread is a pointer move — no allocation in the callback.
//!
//! The executor is the reference [`Vm`], behind an executor seam (`Backend`) a
//! compiled backend can slot back into. It fills a block of samples through one
//! [`Engine::process`] call, so the audio shell does not care which is running.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::ir::Program;
use crate::metrics::ReconcileStats;
use crate::reconcile::{
    BufCopy, BufferRegion, BufferSlot, Migrate, Plane, StatefulNode, buffer_copy_plan,
    buffer_layout, buffer_regions, can_donate, plan, signatures,
};
use crate::vm::Vm;

/// One render block bound, in interleaved f32s, shared by every driver's chunking and by the
/// host's scratch buffers (including [`EngineHost`]'s fade scratch) — so a block of any channel
/// width chunks to fit and no path ever allocates per render.
pub(crate) const MAX_BLOCK: usize = 4096;

/// Running total of migration entries skipped at adoption because their indices fell out of
/// range (see [`Engine::adopt_state_from`]). A bounded, monotonic diagnostic: the audio
/// thread only does a relaxed increment (realtime-safe), and the control thread reads it off
/// the realtime path via [`skipped_migrations`]. It stays at zero in correct operation — a
/// nonzero value means a reconciliation plan was built against the wrong predecessor.
static SKIPPED_MIGRATIONS: AtomicU32 = AtomicU32::new(0);

/// The running total of reconciliation migration entries skipped due to a bounds mismatch.
pub fn skipped_migrations() -> u32 {
    SKIPPED_MIGRATIONS.load(Ordering::Relaxed)
}

/// Device-boundary speaker protection lives in [`crate::sanitize`]; re-exported here so the
/// realtime shells keep calling `engine::sanitize_block` and the REPL `engine::nonfinite_zapped`.
pub use crate::sanitize::{nonfinite_zapped, sanitize_block};

/// The realtime hot-swap loop [`EngineHost`] lives in [`crate::enginehost`]; re-exported so
/// `engine::EngineHost` keeps resolving for `host`, `wasm`, and the harness.
pub use crate::enginehost::EngineHost;

/// Which executor runs the program. Picked on the control thread at build time.
enum Backend {
    /// The reference interpreter.
    Vm(Vm),
}

pub struct Engine {
    program: Program,
    backend: Backend,
    /// State copies to apply when this engine is adopted, carrying state from the engine
    /// it displaces (see [`crate::reconcile`]). Empty means a clean start.
    migration: Vec<Migrate>,
    /// Whether to donate the displaced engine's buffer arena into this one on adoption — set
    /// on the control thread when the buffer layouts match (see [`crate::reconcile::buffer_layout`]),
    /// so delay-line contents survive an edit. `false` starts the buffers fresh-zero.
    donate_buffers: bool,
    /// Per-buffer copies to apply on a *faded* adoption (see [`crate::reconcile::buffer_copy_plan`]).
    /// A faded swap cannot donate — the displaced engine keeps using (and writing) its own arena
    /// for the fade window — so matched regions are copied instead, bounded on the control thread.
    buffer_copies: Vec<BufCopy>,
    /// Crossfade window, in frames, this engine requests when adopted: [`EngineHost::swap_to`]
    /// retains the displaced engine and mixes it out linearly over this many frames. 0 = instant
    /// (today's swap, bit-for-bit).
    fade_frames: u64,
}

impl Engine {
    /// Build an engine that runs `program` on the reference VM.
    pub fn new(program: Program) -> Self {
        let backend = Backend::Vm(Vm::new(&program));
        Engine { program, backend, migration: Vec::new(), donate_buffers: false, buffer_copies: Vec::new(), fade_frames: 0 }
    }

    /// Attach the migration plan to apply on adoption (built on the control thread by
    /// [`crate::reconcile::plan`] against the engine this one will displace).
    pub fn with_migration(mut self, migration: Vec<Migrate>) -> Self {
        self.migration = migration;
        self
    }

    /// Mark whether to donate the displaced engine's buffer arena on adoption (the buffer
    /// layouts matched — see [`crate::reconcile::buffer_layout`]), so delay lines survive the
    /// edit. Built on the control thread alongside the migration plan.
    pub fn with_buffer_donation(mut self, donate: bool) -> Self {
        self.donate_buffers = donate;
        self
    }

    /// Attach the per-buffer copy plan a *faded* adoption applies instead of donation (built
    /// on the control thread by [`crate::reconcile::buffer_copy_plan`], budget-capped there).
    pub fn with_buffer_copies(mut self, copies: Vec<BufCopy>) -> Self {
        self.buffer_copies = copies;
        self
    }

    /// Ask for a crossfade over `frames` when this engine is adopted (0 = instant swap).
    pub fn with_fade_frames(mut self, frames: u64) -> Self {
        self.fade_frames = frames;
        self
    }

    /// The crossfade window this engine requests on adoption (0 = instant), read by
    /// [`EngineHost::swap_to`](crate::enginehost::EngineHost::swap_to).
    pub(crate) fn fade_frames(&self) -> u64 {
        self.fade_frames
    }

    /// Whether this engine's buffer arena may be donated on an instant swap (the layouts
    /// matched), read by [`EngineHost::swap_to`](crate::enginehost::EngineHost::swap_to).
    pub(crate) fn donate_buffers(&self) -> bool {
        self.donate_buffers
    }

    /// The persistent per-UGen state buffer of the running backend.
    pub fn state(&self) -> &[f32] {
        match &self.backend {
            Backend::Vm(vm) => vm.state(),
        }
    }

    /// The feedback-bus buffer of the running backend (its own plane).
    pub fn buses(&self) -> &[f32] {
        match &self.backend {
            Backend::Vm(vm) => vm.buses(),
        }
    }

    /// The sample-memory (buffer) arena of the running backend.
    pub fn buffers(&self) -> &[f32] {
        match &self.backend {
            Backend::Vm(vm) => vm.buffers(),
        }
    }

    /// The buffer arena's backing `Vec`, for whole-arena donation.
    fn buffers_vec_mut(&mut self) -> &mut Vec<f32> {
        match &mut self.backend {
            Backend::Vm(vm) => vm.buffers_vec_mut(),
        }
    }

    /// Donate `old`'s buffer arena into this engine: an O(1) swap of the two backing `Vec`s.
    /// This engine (about to become current) takes `old`'s populated delay lines; `old` (about
    /// to be recycled on the control thread) takes this engine's fresh-zero arena. No content
    /// copy, no allocation — realtime-safe. The caller guarantees the lengths match.
    pub fn donate_buffers_from(&mut self, old: &mut Engine) {
        std::mem::swap(self.buffers_vec_mut(), old.buffers_vec_mut());
    }

    /// Carry state from `old` (the engine being displaced) into this one, per the
    /// migration plan. Realtime-safe: a bounded list of slice copies, no allocation.
    /// Called on the audio thread at adoption, before the first [`Engine::process`].
    /// Each migration targets its own plane's arena (state or bus); buffers are carried
    /// separately by donation (see [`EngineHost::swap_to`]), not copied here.
    pub fn adopt_state_from(&mut self, old: &Engine) {
        let old_state = old.state();
        let old_buses = old.buses();
        // Borrow both arenas directly off the backend in one match, so this stays a
        // field-level borrow disjoint from the immutable `migration` borrow below.
        let (new_state, new_buses) = match &mut self.backend {
            Backend::Vm(vm) => vm.state_and_buses_mut(),
        };
        for m in &self.migration {
            let (o, n, len) = (m.old_base as usize, m.new_base as usize, m.slots as usize);
            let (src, dst): (&[f32], &mut [f32]) = match m.plane {
                Plane::State => (old_state, &mut *new_state),
                Plane::Bus => (old_buses, &mut *new_buses),
            };
            // Guard the indices: the plan is built against the engine actually displaced,
            // so these hold — but a panic here would be on the audio thread, so skip a
            // mismatch rather than abort the stream, and count it so the control thread can
            // notice a plan that did not fully apply (see [`skipped_migrations`]).
            if o + len <= src.len() && n + len <= dst.len() {
                // Element-wise, zeroing non-finite values (= the slot's fresh init): a NaN
                // that poisoned a filter's memory must not outlive the edit — migrating it
                // would make the patch unhealable by re-evaluation.
                for (d, &s) in dst[n..n + len].iter_mut().zip(&src[o..o + len]) {
                    *d = if s.is_finite() { s } else { 0.0 };
                }
            } else {
                SKIPPED_MIGRATIONS.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Copy buffer regions from `old` per this engine's bounded copy plan — the faded-swap
    /// analogue of donation ([`Engine::donate_buffers_from`]): the displaced engine keeps
    /// audibly using and writing its own arena for the fade window, so the arena cannot move
    /// and matched regions are copied instead. After the copy both engines write identical
    /// samples into their respective arenas, which is what keeps the crossfade cancelling on
    /// unchanged delay lines. Realtime-safe: a pre-bounded list of slice copies (budget-capped
    /// on the control thread), bounds guarded like [`Engine::adopt_state_from`].
    pub fn adopt_buffers_from(&mut self, old: &Engine) {
        let src = old.buffers();
        let dst: &mut Vec<f32> = match &mut self.backend {
            Backend::Vm(vm) => vm.buffers_vec_mut(),
        };
        for c in &self.buffer_copies {
            let (o, n, len) = (c.old_base as usize, c.new_base as usize, c.len as usize);
            if o + len <= src.len() && n + len <= dst.len() {
                // Element-wise like `adopt_state_from`, zeroing non-finite samples so a NaN
                // recirculating in a feedback line does not survive the edit. (Fade-0 buffer
                // *donation* stays an O(1) Vec swap and can still carry one — the next faded
                // or structural edit heals it.)
                for (d, &s) in dst[n..n + len].iter_mut().zip(&src[o..o + len]) {
                    *d = if s.is_finite() { s } else { 0.0 };
                }
            } else {
                SKIPPED_MIGRATIONS.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// The number of output channels (≥ 1) — audio channels plus observation taps. `process`
    /// writes frames of this width; the host routes the leading [`audio_channels`](Self::audio_channels)
    /// to the device and meters the rest.
    pub fn channels(&self) -> usize {
        self.program.outputs().len()
    }

    /// Audio output channels routed to the device (≤ [`channels`](Self::channels); the trailing
    /// registers are observation taps the host meters but never plays).
    pub fn audio_channels(&self) -> usize {
        self.program.audio_channels()
    }

    /// Names of this engine's observation taps, aligned with the outputs past `audio_channels`.
    pub fn tap_names(&self) -> &[String] {
        self.program.tap_names()
    }

    /// Audio input channels the program reads; `input` passed to [`Engine::process`] must
    /// be this wide per frame (the host zero-pads to it).
    pub fn in_channels(&self) -> usize {
        self.program.in_channels()
    }

    /// Control-plane width the program reads; `control` passed to [`Engine::process`] must be
    /// at least this wide. The host writes the MIDI-derived values here, latched per block.
    pub fn control_len(&self) -> usize {
        self.program.control_len()
    }

    /// Voice-pool size (1 = monophonic); the host spreads MIDI notes across this many voices.
    pub fn voice_count(&self) -> usize {
        self.program.voice_count()
    }

    /// Fill `out` from the start of the global clock (frame position 0). The entry for callers
    /// that do not track absolute time — tests, the harness, a one-shot render. The realtime
    /// host instead calls [`process_from`](Self::process_from) so `now` advances across blocks.
    pub fn process(&mut self, input: &[f32], control: &[f32], out: &mut [f32]) {
        self.process_from(0, input, control, out);
    }

    /// Fill `out` with the next frames, interleaved by channel: `out.len()` must be a
    /// multiple of [`Engine::channels`]. `block_start_pos` is the global sample-clock position
    /// of the first frame (the executor advances it one per frame, surfacing to UGens as `now`);
    /// the engine reads it but never owns it — the canonical clock lives on [`EngineHost`].
    /// `input` is read interleaved, `in_channels` wide per frame (`input.len() == (out.len() /
    /// channels) * in_channels`). `control` is the per-block MIDI plane (`control_len` wide), held
    /// constant for the whole call (the same slice feeds every frame — the plane is
    /// frame-invariant). Advances state.
    pub fn process_from(&mut self, block_start_pos: u64, input: &[f32], control: &[f32], out: &mut [f32]) {
        let channels = self.program.outputs().len();
        let in_ch = self.program.in_channels();
        // The host must zero-pad `input` to `frames * in_channels` (see the doc above);
        // make the contract a checked invariant.
        debug_assert!(
            input.len() >= (out.len() / channels) * in_ch,
            "input block too short: {} < {} frames * {in_ch} in_channels",
            input.len(),
            out.len() / channels
        );
        debug_assert!(
            control.len() >= self.program.control_len(),
            "control block too short: {} < {} control_len",
            control.len(),
            self.program.control_len()
        );
        match &mut self.backend {
            Backend::Vm(vm) => {
                // The control plane is frame-invariant (latched per block), so the same
                // `control` slice feeds every frame; the clock position advances per frame.
                for (f, frame) in out.chunks_mut(channels).enumerate() {
                    vm.tick_frame(
                        &self.program,
                        block_start_pos + f as u64,
                        &input[f * in_ch..f * in_ch + in_ch],
                        control,
                        frame,
                    );
                }
            }
        }
    }
}

/// The reconciliation bookkeeping a frontend carries between hot-swaps: the signatures,
/// buffer layout, and buffer regions of the last engine actually handed to the driver —
/// the one the next edit migrates against. [`stage`](Carryover::stage) plans a swap
/// against it and returns the successor bookkeeping in [`Staged::next`]; the caller
/// assigns that over this value only once the hand-off succeeded, so a dropped edit is
/// never adopted.
#[derive(Default)]
pub struct Carryover {
    sig: Vec<StatefulNode>,
    buf: Vec<BufferSlot>,
    regions: Vec<BufferRegion>,
}

/// A hot-swap staged by [`Carryover::stage`]: the engine to hand to the driver, the
/// fade-copy declines to surface, and the bookkeeping to adopt on success.
pub struct Staged {
    /// The engine, with its migration plan, fade window, and buffer carry attached.
    pub engine: Engine,
    /// Matched buffer regions the faded copy plan declined over budget (always 0 on an
    /// instant swap) — they reset to silence; both frontends surface the count as a warning.
    pub declined: u32,
    /// The reconciliation report for this swap — how much of the running engine carried over.
    /// Surfaced as metrics by both frontends (it never affects the swap itself).
    pub reconcile: ReconcileStats,
    /// The successor bookkeeping: assign it over the [`Carryover`] iff the engine was
    /// handed over (native: the ring send succeeded; single-threaded drivers: always).
    pub next: Carryover,
}

impl Carryover {
    /// Stage `engine` for a hot-swap against the last committed program: attach the state
    /// migration plan ([`crate::reconcile::plan`]) and the fade, and carry buffers — donation
    /// when the swap is instant and the layout is unchanged *and* unambiguous
    /// ([`crate::reconcile::can_donate`]; declines indistinguishable buffers rather than risk
    /// cross-swapping on a reorder), or the budget-capped per-region copy plan when it fades
    /// ([`crate::reconcile::buffer_copy_plan`]; the displaced engine keeps using its arena
    /// while it fades). `fade_seconds` converts at the program's sample rate.
    pub fn stage(&self, engine: Engine, fade_seconds: f32) -> Staged {
        let program = &engine.program;
        let sig = signatures(program);
        let buf = buffer_layout(program);
        let regions = buffer_regions(program);
        let fade_frames = (fade_seconds.max(0.0) * program.sample_rate) as u64;
        // The state/bus migration plan, computed once: its length is how many stateful ops carried
        // over, the sum of its slot counts how many scalars — both surfaced as reconcile metrics
        // before the plan moves onto the engine.
        let migration = plan(&self.sig, &sig);
        let scalars_carried: u32 = migration.iter().map(|m| m.slots).sum();
        let mut stats = ReconcileStats {
            stateful_total: sig.len() as u32,
            stateful_reused: migration.len() as u32,
            scalars_carried,
            buffers_total: buf.len() as u32,
            ..ReconcileStats::default()
        };
        let mut next = engine.with_migration(migration).with_fade_frames(fade_frames);
        let mut declined = 0;
        next = if fade_frames == 0 {
            // Instant swap: the whole buffer arena is donated (O(1)) when the layout is unchanged.
            let donate = can_donate(&self.buf, &buf);
            stats.buffers_donated = if donate { buf.len() as u32 } else { 0 };
            next.with_buffer_donation(donate)
        } else {
            // Faded swap: matched regions are copied per the budget-capped plan.
            let bp = buffer_copy_plan(&self.regions, &regions);
            declined = bp.declined;
            stats.regions_copied = bp.copies.len() as u32;
            stats.regions_declined = bp.declined;
            next.with_buffer_copies(bp.copies)
        };
        Staged { engine: next, declined, reconcile: stats, next: Carryover { sig, buf, regions } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::compile;
    use crate::graph::Graph;
    use crate::testutil::{graph_of, u};

    fn engine(graph: &Graph) -> Engine {
        Engine::new(compile(graph, 48_000.0))
    }

    #[test]
    fn sanitize_block_zaps_non_finite_and_counts() {
        // `1 0 /` is +inf every sample (documented IEEE behavior) — the device boundary must
        // replace it with silence and count what it zapped, leaving finite samples untouched.
        let mut e = engine(&graph_of(|g| {
            let a = g.constant(1.0);
            let b = g.constant(0.0);
            vec![g.ugen(u("/"), vec![a, b])] // `1 0 /` — +inf every sample
        }));
        let mut out = [0.0_f32; 64];
        e.process(&[], &[], &mut out);
        assert!(out.iter().all(|s| s.is_infinite()), "the raw engine output is IEEE inf");
        let before = nonfinite_zapped();
        sanitize_block(&mut out);
        assert!(out.iter().all(|&s| s == 0.0), "non-finite samples become silence");
        assert_eq!(nonfinite_zapped(), before + 64, "every zapped sample is counted");

        let mut fine = [0.25_f32; 8];
        sanitize_block(&mut fine);
        assert!(fine.iter().all(|&s| s == 0.25), "finite samples pass through untouched");
        assert_eq!(nonfinite_zapped(), before + 64, "nothing further was counted");
    }

    #[test]
    fn migration_scrubs_non_finite_state_so_an_edit_heals() {
        use crate::reconcile::{plan, signatures};
        // `noise 0 * 0 /` is NaN every sample; the lpf integrates it into a sticky-NaN state.
        let poisoned = || {
            graph_of(|g| {
                let n = g.ugen(u("noise"), vec![]);
                let z1 = g.constant(0.0);
                let m = g.ugen(u("*"), vec![n, z1]);
                let z2 = g.constant(0.0);
                let d = g.ugen(u("/"), vec![m, z2]);
                let c = g.constant(100.0);
                vec![g.ugen(u("lpf"), vec![d, c])]
            })
        };
        let prog_a = compile(&poisoned(), 48_000.0);
        let prog_b = compile(&poisoned(), 48_000.0);
        let migration = plan(&signatures(&prog_a), &signatures(&prog_b));
        let mut a = Engine::new(prog_a);
        a.process(&[], &[], &mut [0.0; 64]);
        assert!(a.state().iter().any(|s| s.is_nan()), "the filter state is poisoned");
        assert!(a.state().iter().any(|s| s.is_finite() && *s != 0.0), "the noise counter ran");

        // Adopting across a re-eval migrates the finite slots and zeroes the poisoned one —
        // the edit heals the patch instead of carrying the NaN forward.
        let mut b = Engine::new(prog_b).with_migration(migration);
        b.adopt_state_from(&a);
        assert!(b.state().iter().all(|s| s.is_finite()), "no NaN survives the migration");
        assert!(
            b.state().iter().any(|&s| s != 0.0),
            "the finite slots (the noise counter) still migrate"
        );
    }

    #[test]
    fn input_passes_through_to_output() {
        // `in` reads the current frame's input channel 0, so `in out` is a wire from the
        // input block to the output. Feeding a known block must reproduce it exactly.
        let p = compile(&graph_of(|g| vec![g.input(0)]), 48_000.0);
        let mut e = Engine::new(p);
        let input = [0.1_f32, 0.2, 0.3, 0.4];
        let mut out = [0.0_f32; 4];
        e.process(&input, &[], &mut out);
        assert_eq!(out, input);
    }

}
