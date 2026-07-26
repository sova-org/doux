//! User-defined arf graphs as doux voice sources (`s/<name>`), voice
//! inserts (`fx/<name>`) and orbit effects (`patch/<name>`).
//!
//! One namespace serves all three roles; the role is determined by the
//! program itself: `in_channels() == 0` is a source, `>= 1` an effect
//! (see [`PatchEntry::is_source`] / [`PatchEntry::is_effect`]). Use-sites
//! enforce the role — install only enforces the shared caps.
//!
//! Split by thread: [`PatchRegistry::install_graph`] and [`PatchRegistry::install`] run on a
//! control thread (they deserialize, compile, validate and pre-build a [`Vm`] pool). The audio thread
//! only does lock-free work: [`PatchRegistry::get`], [`PatchEntry::take_vm`] at
//! note-on, and [`PatchRegistry::retire`] when a voice ends — on native the
//! dirty `Vm` crosses a channel to the patch-reaper thread, which resets it
//! off-RT and pushes it back into its pool.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

use arc_swap::ArcSwap;
use arf::graph::{BPS_LANE, CONTROL_WIDTH, MAX_PARAMS, PARAM_BASE};
use arf::ir::Program;
use arf::vm::Vm;
use crossbeam_queue::ArrayQueue;

/// Default Vm-pool depth when a registry is built without an explicit
/// polyphony ([`PatchRegistry::new`], used by tests and external callers):
/// matches doux's default voice ceiling so an arf source is never *more*
/// polyphony-limited than a native voice. [`PatchRegistry::with_polyphony`]
/// overrides it to track a custom `max_voices` — including a lower ceiling on
/// constrained devices, where a big pool would waste memory. An event that
/// finds the pool dry is still dropped.
const DEFAULT_POOL_VMS: usize = crate::types::DEFAULT_MAX_VOICES;
/// Per-Vm sample-memory cap in f32s (4 MiB). One default `delay` line is
/// 2^16 f32s, so this allows 16 per patch. Provisional (to_do.md session 5).
const MAX_BUFFER_LEN: usize = 1 << 20;
/// Install-time cap on [`Program::weight`] (a bare sine weighs ~12) — a
/// guard against runaway generated graphs, not a perf promise.
/// Provisional (to_do.md session 5).
const MAX_GRAPH_WEIGHT: u32 = 4096;
/// The tempo a patch reads from its transport lane before the engine's first
/// block write: 2 beats/s = 120 BPM, so `bps` is never silently 0.
const DEFAULT_BPS: f32 = 2.0;
/// Depth of the native Vm-return channel: comfortably above any burst of
/// voice deaths in one block (mirrors the event reaper's headroom).
#[cfg(feature = "native")]
const VM_RETURN_DEPTH: usize = 256;

/// An installed patch: the shared program and its pre-built [`Vm`] pool.
pub struct PatchEntry {
    program: Arc<Program>,
    pool: ArrayQueue<Vm>,
}

impl PatchEntry {
    pub fn program(&self) -> &Arc<Program> {
        &self.program
    }

    /// Pop a ready (reset, seeded) Vm. Lock-free, allocation-free — RT-safe.
    pub(crate) fn take_vm(&self) -> Option<Vm> {
        self.pool.pop()
    }

    /// Lock-free pool probe, used by the event availability gate so a dry
    /// pool drops the event before a voice slot is spent on silence.
    pub(crate) fn has_vm(&self) -> bool {
        !self.pool.is_empty()
    }

    /// A source generates audio from nothing — playable as `s/<name>`.
    pub(crate) fn is_source(&self) -> bool {
        self.program.in_channels() == 0
    }

    /// An effect reads audio input — usable as `fx/<name>` or `patch/<name>`.
    pub(crate) fn is_effect(&self) -> bool {
        self.program.in_channels() > 0
    }
}

/// A live handle on a patch — voice source, voice insert, or orbit effect:
/// the entry (for the program and the way home to its pool), the running
/// [`Vm`], the local sample clock, and the control plane. The source loop
/// writes gate/notefreq/vel; every role gets the engine tempo latched into
/// [`BPS_LANE`] each block (effects may read nothing else). Named-param
/// lanes start at their declared defaults.
pub struct VoicePatch {
    pub(crate) entry: Arc<PatchEntry>,
    pub(crate) vm: Vm,
    pub(crate) frame_pos: u64,
    pub(crate) control: [f32; CONTROL_WIDTH],
}

impl VoicePatch {
    pub(crate) fn new(entry: Arc<PatchEntry>, vm: Vm) -> Self {
        let mut control = [0.0; CONTROL_WIDTH];
        control[BPS_LANE] = DEFAULT_BPS;
        for (i, (_, default)) in entry.program().params().iter().enumerate() {
            control[PARAM_BASE + i] = *default;
        }
        VoicePatch {
            entry,
            vm,
            frame_pos: 0,
            control,
        }
    }
}

/// Zero any non-finite sample in a patch's output frame, returning `true` if it
/// zeroed anything. arf's core is IEEE-transparent (a pathological graph can
/// emit NaN/inf); doux scrubs at the boundary so one bad patch cannot poison
/// downstream filter state. The `bool` lets the tick sites flag a poisoned Vm
/// for the NaN-heal path (whose own state may still be latched even after the
/// output frame is scrubbed). Shared by all three tick sites (source, insert,
/// orbit effect).
#[inline]
pub(crate) fn scrub_non_finite(frame: &mut [f32]) -> bool {
    let mut bad = false;
    for s in frame {
        if !s.is_finite() {
            *s = 0.0;
            bad = true;
        }
    }
    bad
}

/// A message to the patch-reaper thread.
#[cfg(feature = "native")]
enum Reap {
    /// A dirty Vm coming home: reset it off-RT and push it back into its
    /// entry's pool.
    Vm(Arc<PatchEntry>, Vm),
    /// An entry displaced by a reinstall. The reaper parks it in its
    /// graveyard so a transient audio-thread clone (dispatch gate / attach)
    /// can never become the entry's last owner — the final multi-MiB free of
    /// a whole Vm pool must happen off-RT.
    Entry(Arc<PatchEntry>),
}

/// Lock-free name → patch registry, the arf mirror of
/// [`crate::sampling::SampleRegistry`]: writers swap a rebuilt map in
/// atomically, the audio thread reads a consistent snapshot.
pub struct PatchRegistry {
    patches: ArcSwap<HashMap<String, Arc<PatchEntry>>>,
    /// Monotonic `Vm::reset` seed so concurrent voices of one patch get
    /// decorrelated noise. Starts at 1 — offset 0 is the shared baseline.
    next_seed: Arc<AtomicU32>,
    /// Return lane to the "doux-patch-reaper" thread, which resets dirty Vms
    /// off-RT. `None` (spawn failure) degrades to dropping the Vm in place.
    #[cfg(feature = "native")]
    vm_return: Option<crossbeam_channel::Sender<Reap>>,
    /// Vms pre-built per installed patch — the per-patch polyphony ceiling.
    /// Sized to the engine's voice count so an arf source matches native
    /// polyphony (see [`DEFAULT_POOL_VMS`]). Atomic because the host may
    /// retune its voice ceiling live ([`PatchRegistry::set_polyphony`]); a
    /// pool is sized once at install, so a change lands patch by patch as
    /// each is next installed rather than resizing pools already in use.
    pool_vms: AtomicUsize,
}

impl Default for PatchRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PatchRegistry {
    pub fn new() -> Self {
        Self::with_polyphony(DEFAULT_POOL_VMS)
    }

    /// Build a registry whose per-patch Vm pools hold `pool_vms` voices — pass
    /// the engine's `max_voices` so an arf source is neither more
    /// polyphony-limited than a native voice nor over-allocated below it.
    pub fn with_polyphony(pool_vms: usize) -> Self {
        let next_seed = Arc::new(AtomicU32::new(1));
        #[cfg(feature = "native")]
        let vm_return = {
            let (tx, rx) = crossbeam_channel::bounded::<Reap>(VM_RETURN_DEPTH);
            let seed = Arc::clone(&next_seed);
            std::thread::Builder::new()
                .name("doux-patch-reaper".into())
                .spawn(move || {
                    // Displaced entries wait here until nothing else holds
                    // them. Exits when the registry (the last sender) drops.
                    let mut graveyard: Vec<Arc<PatchEntry>> = Vec::new();
                    while let Ok(msg) = rx.recv() {
                        match msg {
                            Reap::Vm(entry, mut vm) => {
                                vm.reset(&entry.program, seed.fetch_add(1, Ordering::Relaxed));
                                // Push can only fail if the entry was
                                // reinstalled and this Vm belongs to an
                                // orphaned pool — drop it here, off-RT.
                                let _ = entry.pool.push(vm);
                            }
                            Reap::Entry(entry) => graveyard.push(entry),
                        }
                        // A graveyard entry at strong count 1 is unreachable
                        // from anywhere else (it left the map, and clones can
                        // only be minted from live refs), so freeing it here
                        // — off-RT — is safe and final.
                        graveyard.retain(|e| Arc::strong_count(e) > 1);
                    }
                })
                .ok()
                .map(|_| tx)
        };
        PatchRegistry {
            patches: ArcSwap::from_pointee(HashMap::new()),
            next_seed,
            #[cfg(feature = "native")]
            vm_return,
            pool_vms: AtomicUsize::new(pool_vms.max(1)),
        }
    }

    /// Retune the per-patch pool depth after construction — the path a host takes when the
    /// user changes the voice ceiling mid-session. Pools are allocated at install, so this
    /// governs installs from here on; patches already resident keep the depth they were
    /// built with until something reinstalls them (any edit does).
    pub fn set_polyphony(&self, pool_vms: usize) {
        self.pool_vms.store(pool_vms.max(1), Ordering::Relaxed);
    }

    /// Validate `program` and publish it under `name` with a ready Vm pool.
    /// Allocates the pool — never call on the audio thread. Reinstalling a
    /// name swaps in a fresh entry; voices still holding the old one keep
    /// playing it and return their Vms to the orphaned pool, which frees
    /// once the last handle drops.
    pub fn install(&self, name: &str, program: Program) -> Result<(), String> {
        if name.is_empty() || name.contains('/') || name.contains(char::is_whitespace) {
            return Err(format!(
                "patch name {name:?} cannot be triggered: it must be non-empty, without '/' or whitespace"
            ));
        }
        if name == "off" {
            return Err(
                "patch name \"off\" is reserved: `patch/off` and `fx/off` clear the slot".into(),
            );
        }
        let channels = program.audio_channels();
        if !(1..=2).contains(&channels) {
            return Err(format!(
                "patch has {channels} audio channels; a doux voice source is mono or stereo"
            ));
        }
        if program.in_channels() > 2 {
            return Err(format!(
                "patch reads {} input channels; doux buses are stereo (max 2)",
                program.in_channels()
            ));
        }
        // A lane past the plane would panic slicing the fixed per-voice control
        // array on the audio thread — reject hand-crafted graph JSON here instead.
        if program.control_len() > CONTROL_WIDTH {
            return Err(format!(
                "patch reads control lane {} (the control plane is {CONTROL_WIDTH} lanes wide)",
                program.control_len() - 1
            ));
        }
        // Same audio-thread guarantee for the name→lane map: `param_lane` yields
        // `PARAM_BASE + index`, so the declaration count must fit the plane too.
        if program.params().len() > MAX_PARAMS {
            return Err(format!(
                "patch declares {} params (cap {MAX_PARAMS})",
                program.params().len()
            ));
        }
        if program.buffer_len() > MAX_BUFFER_LEN {
            return Err(format!(
                "patch needs {} KB of sample memory per voice (cap {} KB)",
                program.buffer_len() * 4 / 1024,
                MAX_BUFFER_LEN * 4 / 1024
            ));
        }
        let weight = program.weight();
        if weight > MAX_GRAPH_WEIGHT {
            return Err(format!(
                "patch weighs {weight} (cap {MAX_GRAPH_WEIGHT}); split it or thin it out"
            ));
        }
        // Per-op checks the summary counts above can't catch. Buffers: arf's
        // compiler lays the arena out in u32; enough max-length named buffers
        // can wrap the cursor so `buffer_len()` passes the cap above while an
        // op still points past the arena the Vm allocates — an out-of-bounds
        // slice panic on the first tick, on the audio thread. Control: an
        // effect's plane carries only the transport tempo — a per-note lane
        // (gate/notefreq/vel) or a named param would silently read a constant,
        // so reject it here instead.
        for op in program.ops() {
            match *op {
                // Arity belt for the direct `install(name, Program)` path (graph JSON is caught
                // earlier by `Graph::validate`): a bad input count would index the VM's fixed
                // `scratch[..input_count]` out of range, or read `ctx.inputs[k]` past the gathered
                // slice inside a `Fixed(n)` tick — an audio-thread panic.
                arf::ir::Op::Ugen {
                    def, input_count, ..
                } if input_count as usize > arf::graph::MAX_CHANNELS
                    || matches!(def.arity, arf::ugen::Arity::Fixed(a) if a != input_count as usize) =>
                {
                    return Err(format!(
                        "patch generator {:?} has {input_count} inputs (bad arity)",
                        def.name
                    ));
                }
                arf::ir::Op::Ugen {
                    buffer_base,
                    buffer_len,
                    ..
                } if buffer_base as u64 + buffer_len as u64 > program.buffer_len() as u64 => {
                    return Err(
                        "patch buffer layout overflows its arena (u32 wrap in a buffer size sum)"
                            .to_string(),
                    );
                }
                arf::ir::Op::Control { lane }
                    if program.in_channels() > 0 && lane as usize != BPS_LANE =>
                {
                    return Err(
                        "effect patch reads a per-note control lane (gate/notefreq/vel/param); \
                         effects may read only `bps`"
                            .to_string(),
                    );
                }
                _ => {}
            }
        }

        let program = Arc::new(program);
        // Latched once: the pool and the fill loop must agree even if the host retunes
        // polyphony between the two.
        let pool_vms = self.pool_vms.load(Ordering::Relaxed);
        let pool = ArrayQueue::new(pool_vms);
        for _ in 0..pool_vms {
            let mut vm = Vm::new(&program);
            vm.reset(&program, self.next_seed.fetch_add(1, Ordering::Relaxed));
            let _ = pool.push(vm);
        }
        let entry = Arc::new(PatchEntry {
            program: Arc::clone(&program),
            pool,
        });
        // A reinstall displaces the old entry. Park it in the reaper's
        // graveyard so it always has a durable off-RT owner: without one, a
        // transient audio-thread clone (dispatch gate racing this swap) could
        // end up freeing the entire old Vm pool inside the callback. If the
        // reaper is gone the entry drops right here — this is a control
        // thread, so that is still off-RT.
        #[cfg(feature = "native")]
        let displaced = self.patches.load().get(name).cloned();
        self.patches.rcu(|cur| {
            let mut map = HashMap::clone(cur);
            map.insert(name.to_string(), Arc::clone(&entry));
            Arc::new(map)
        });
        #[cfg(feature = "native")]
        if let (Some(old), Some(tx)) = (displaced, &self.vm_return) {
            let _ = tx.send(Reap::Entry(old));
        }
        Ok(())
    }

    /// Compile a serialized arf graph at this engine's sample rate and install it under
    /// `name`. The graph JSON is the language-agnostic patch boundary — a front-end (cagire's
    /// `arf-forth`, or any other) builds an `arf::graph::Graph`, serializes it, and hands it
    /// here. doux plays graphs; it never parses a patch language. Allocates — control thread only.
    pub fn install_graph(&self, name: &str, graph_json: &str, sr: f32) -> Result<(), String> {
        let graph: arf::graph::Graph =
            serde_json::from_str(graph_json).map_err(|e| format!("invalid patch graph: {e}"))?;
        graph
            .validate()
            .map_err(|e| format!("invalid patch graph: {e}"))?;
        let program = arf::compile::compile(&graph, sr);
        self.install(name, program)
    }

    /// Evict an installed patch by `name`, routing its entry to the reaper's
    /// graveyard for the off-RT free — the same path a reinstall's displaced
    /// entry takes. A voice still sounding on it keeps its own `Arc` clone and
    /// plays out; only the map slot is dropped now, so eviction can never cut a
    /// note. No-op if the name is absent. Control thread only (rebuilds the map
    /// like `install`); the caller owns the eviction policy (cagire's
    /// `PatchInstaller`, which is the single writer of the install/evict ledger
    /// and so keeps its own dedup cache consistent with this map).
    pub fn remove(&self, name: &str) {
        #[cfg(feature = "native")]
        let displaced = self.patches.load().get(name).cloned();
        self.patches.rcu(|cur| {
            let mut map = HashMap::clone(cur);
            map.remove(name);
            Arc::new(map)
        });
        // Native: hand the displaced entry to the reaper so the final multi-MiB
        // pool free happens off-RT. Wasm is single-threaded and this is the
        // control path, so the `rcu` drop above is already off-RT.
        #[cfg(feature = "native")]
        if let (Some(old), Some(tx)) = (displaced, &self.vm_return) {
            let _ = tx.send(Reap::Entry(old));
        }
    }

    /// Look a patch up by its bare name. Lock-free — RT-safe.
    #[inline]
    pub fn get(&self, name: &str) -> Option<Arc<PatchEntry>> {
        self.patches.load().get(name).cloned()
    }

    /// Return a voice's Vm to its patch's pool. RT-safe: on native the dirty
    /// Vm is handed to the patch reaper for its off-RT reset; a full channel
    /// degrades to dropping in place (the event-reaper precedent). On wasm
    /// (single-threaded) the reset runs inline.
    pub(crate) fn retire(&self, patch: VoicePatch) {
        let VoicePatch { entry, vm, .. } = patch;
        self.retire_vm(entry, vm);
    }

    /// Retire a bare `(entry, vm)` pair — the core of [`retire`], reused by the
    /// NaN-heal path in `gen_block`, which swaps a poisoned Vm for a fresh pooled
    /// one and sends the old one home here. RT-safe.
    pub(crate) fn retire_vm(&self, entry: Arc<PatchEntry>, vm: Vm) {
        #[cfg(feature = "native")]
        {
            // On Full/Disconnected — or when the reaper never spawned — the
            // pair drops right here. That frees ONE Vm on the audio thread
            // (the degradation the event reaper also accepts) and no more:
            // the entry ref cannot be the last owner, because the map holds
            // one while the patch is current and the reaper's graveyard
            // holds one after a reinstall displaces it.
            if let Some(tx) = &self.vm_return {
                let _ = tx.try_send(Reap::Vm(entry, vm));
            }
        }
        #[cfg(not(feature = "native"))]
        {
            let mut vm = vm;
            vm.reset(
                &entry.program,
                self.next_seed.fetch_add(1, Ordering::Relaxed),
            );
            let _ = entry.pool.push(vm);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `param cutoff 400` + `notefreq saw cutoff lpf`, built through the graph
    /// API (the Forth front-end lives outside doux).
    fn param_graph() -> arf::graph::Graph {
        let mut g = arf::graph::Graph::new();
        let lane = g.add_param("cutoff".to_string(), 400.0);
        let cut = g.control(lane);
        let nf = g.control(arf::graph::NOTEFREQ_LANE as u32);
        let saw = g.ugen(arf::ugen::lookup("saw").expect("saw is a ugen"), vec![nf]);
        let filt = g.ugen(
            arf::ugen::lookup("lpf").expect("lpf is a ugen"),
            vec![saw, cut],
        );
        g.set_outputs(vec![filt]);
        g
    }

    // The pool depth is the per-patch polyphony ceiling, so a host that retunes its voice
    // count must be able to move it. It applies at install: the patch resident when the
    // change lands keeps its depth, the next install gets the new one.
    #[test]
    fn set_polyphony_governs_pools_from_the_next_install() {
        let registry = PatchRegistry::with_polyphony(2);
        let json = serde_json::to_string(&param_graph()).unwrap();
        registry.install_graph("small", &json, 48_000.0).unwrap();
        let small = registry.get("small").unwrap();
        assert!(small.take_vm().is_some());
        assert!(small.take_vm().is_some());
        assert!(small.take_vm().is_none(), "pool must hold exactly 2");

        registry.set_polyphony(5);
        registry.install_graph("big", &json, 48_000.0).unwrap();
        let big = registry.get("big").unwrap();
        assert_eq!(
            std::iter::from_fn(|| big.take_vm()).count(),
            5,
            "the next install takes the new depth"
        );
        // A floor of 1, so a host that passes 0 still gets a playable patch.
        registry.set_polyphony(0);
        registry.install_graph("floor", &json, 48_000.0).unwrap();
        assert!(registry.get("floor").unwrap().take_vm().is_some());
    }

    #[test]
    fn params_survive_the_json_boundary_and_fill_voice_defaults() {
        let registry = PatchRegistry::new();
        let json = serde_json::to_string(&param_graph()).unwrap();
        registry.install_graph("pp", &json, 48_000.0).unwrap();

        let entry = registry.get("pp").unwrap();
        assert_eq!(entry.program().params(), &[("cutoff".to_string(), 400.0)]);
        assert_eq!(
            entry.program().param_lane("cutoff"),
            Some(PARAM_BASE as u32)
        );
        assert_eq!(entry.program().param_lane("nope"), None);

        let vm = entry.take_vm().unwrap();
        let vp = VoicePatch::new(entry, vm);
        assert_eq!(vp.control[PARAM_BASE], 400.0);
    }

    #[test]
    fn a_returned_vm_yields_a_fresh_voice_patch_at_defaults() {
        // The pool recycles Vms, but the control plane lives in VoicePatch,
        // built fresh per note — a written lane cannot leak across notes.
        let registry = PatchRegistry::new();
        let json = serde_json::to_string(&param_graph()).unwrap();
        registry.install_graph("pp", &json, 48_000.0).unwrap();

        let entry = registry.get("pp").unwrap();
        let vm = entry.take_vm().unwrap();
        let mut vp = VoicePatch::new(Arc::clone(&entry), vm);
        vp.control[PARAM_BASE] = 9_999.0;
        registry.retire(vp);

        let vm = entry.take_vm().unwrap();
        let vp = VoicePatch::new(entry, vm);
        assert_eq!(vp.control[PARAM_BASE], 400.0);
    }

    #[test]
    fn install_rejects_an_oversized_control_plane() {
        // A hand-crafted graph can read past the fixed per-voice control
        // array; unchecked, that panics on the audio thread.
        let registry = PatchRegistry::new();
        let mut g = arf::graph::Graph::new();
        let lane = g.control(CONTROL_WIDTH as u32);
        g.set_outputs(vec![lane]);
        let json = serde_json::to_string(&g).unwrap();
        let err = registry.install_graph("wide", &json, 48_000.0).unwrap_err();
        assert!(err.contains("control lane"), "unexpected error: {err}");
    }

    #[test]
    fn install_rejects_too_many_params() {
        let registry = PatchRegistry::new();
        let mut g = arf::graph::Graph::new();
        for i in 0..=MAX_PARAMS {
            g.add_param(format!("p{i}"), 0.0);
        }
        let out = g.constant(0.0);
        g.set_outputs(vec![out]);
        let json = serde_json::to_string(&g).unwrap();
        let err = registry.install_graph("many", &json, 48_000.0).unwrap_err();
        assert!(err.contains("params"), "unexpected error: {err}");
    }
}
