//! User-defined arf graphs as doux voice sources (`s/arf:<name>`).
//!
//! Split by thread: [`PatchRegistry::install_graph`] and [`PatchRegistry::install`] run on a
//! control thread (they deserialize, compile, validate and pre-build a [`Vm`] pool). The audio thread
//! only does lock-free work: [`PatchRegistry::get`], [`PatchEntry::take_vm`] at
//! note-on, and [`PatchRegistry::retire`] when a voice ends — on native the
//! dirty `Vm` crosses a channel to the patch-reaper thread, which resets it
//! off-RT and pushes it back into its pool.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use arc_swap::ArcSwap;
use arf::graph::CONTROL_WIDTH;
use arf::ir::Program;
use arf::vm::Vm;
use crossbeam_queue::ArrayQueue;

/// Vms pre-built per installed patch — the per-patch polyphony ceiling; an
/// event that finds the pool dry is dropped. Provisional (to_do.md session 5).
const POOL_VMS: usize = 8;
/// Per-Vm sample-memory cap in f32s (4 MiB). One default `delay` line is
/// 2^16 f32s, so this allows 16 per patch. Provisional (to_do.md session 5).
const MAX_BUFFER_LEN: usize = 1 << 20;
/// Install-time cap on [`arf::metrics::graph_weight`] (a bare sine weighs
/// ~12) — a guard against runaway generated graphs, not a perf promise.
/// Provisional (to_do.md session 5).
const MAX_GRAPH_WEIGHT: u32 = 4096;
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
}

/// A voice's live handle on a patch: the entry (for the program and the way
/// home to its pool), the running [`Vm`], the voice-local sample clock, and
/// the control plane the source loop writes gate/notefreq/vel into.
pub struct VoicePatch {
    pub(crate) entry: Arc<PatchEntry>,
    pub(crate) vm: Vm,
    pub(crate) frame_pos: u64,
    pub(crate) control: [f32; CONTROL_WIDTH],
}

impl VoicePatch {
    pub(crate) fn new(entry: Arc<PatchEntry>, vm: Vm) -> Self {
        VoicePatch {
            entry,
            vm,
            frame_pos: 0,
            control: [0.0; CONTROL_WIDTH],
        }
    }
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
}

impl Default for PatchRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PatchRegistry {
    pub fn new() -> Self {
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
        }
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
        let channels = program.audio_channels();
        if !(1..=2).contains(&channels) {
            return Err(format!(
                "patch has {channels} audio channels; a doux voice source is mono or stereo"
            ));
        }
        if program.in_channels() > 0 {
            return Err(format!(
                "patch reads {} input channels; a voice source takes no audio input (effect patches come later)",
                program.in_channels()
            ));
        }
        if program.voice_count() > 1 {
            return Err(format!(
                "patch declares {} voices; doux owns polyphony — drop `voices`, each note gets its own instance",
                program.voice_count()
            ));
        }
        if program.buffer_len() > MAX_BUFFER_LEN {
            return Err(format!(
                "patch needs {} KB of sample memory per voice (cap {} KB)",
                program.buffer_len() * 4 / 1024,
                MAX_BUFFER_LEN * 4 / 1024
            ));
        }
        let weight = arf::metrics::graph_weight(&program);
        if weight > MAX_GRAPH_WEIGHT {
            return Err(format!(
                "patch weighs {weight} (cap {MAX_GRAPH_WEIGHT}); split it or thin it out"
            ));
        }
        // arf's compiler lays the buffer arena out in u32; enough max-length
        // named buffers can wrap the cursor so `buffer_len()` passes the cap
        // above while an op still points past the arena the Vm allocates —
        // an out-of-bounds slice panic on the first tick, on the audio
        // thread. Check every op's slice against the arena it will index.
        for op in program.ops() {
            if let arf::ir::Op::Ugen {
                buffer_base,
                buffer_len,
                ..
            } = *op
            {
                if buffer_base as u64 + buffer_len as u64 > program.buffer_len() as u64 {
                    return Err(
                        "patch buffer layout overflows its arena (u32 wrap in a buffer size sum)"
                            .to_string(),
                    );
                }
            }
        }

        let program = Arc::new(program);
        let pool = ArrayQueue::new(POOL_VMS);
        for _ in 0..POOL_VMS {
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
    /// `name`. The graph JSON is the language-agnostic patch boundary — a front-end
    /// (`arf-forth`, or any other) builds an `arf::graph::Graph`, serializes it, and hands it
    /// here. doux plays graphs; it never parses a patch language. Allocates — control thread only.
    pub fn install_graph(&self, name: &str, graph_json: &str, sr: f32) -> Result<(), String> {
        let graph: arf::graph::Graph =
            serde_json::from_str(graph_json).map_err(|e| format!("invalid patch graph: {e}"))?;
        let program = arf::compile::compile(&graph, sr);
        self.install(name, program)
    }

    /// Look a patch up by its bare name (no `arf:` prefix). Lock-free — RT-safe.
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
            vm.reset(&entry.program, self.next_seed.fetch_add(1, Ordering::Relaxed));
            let _ = entry.pool.push(vm);
        }
    }
}
