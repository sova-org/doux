//! doux — a real-time software-synthesizer engine for live coding.
//!
//! Builds two ways: a native library + binaries (cdylib/rlib, `native` feature,
//! cpal audio) and a `wasm32-unknown-unknown` module driven from a browser
//! AudioWorklet (`src/wasm.rs`). Both share the same engine core.
//!
//! Module map: `voice` (per-voice synthesis + insert chain), `effects` and its
//! `faust_dsp` submodule (orbit/insert effects, most Faust-generated), `orbit`
//! (Tidal-style persistent FX buses), `event` (parses OSC/eval strings into
//! typed events), `schedule`, `sampling`, `types` (constants + param vocabulary).
//!
//! Hard invariant: the audio render callback (`Engine::process_block`) must
//! never allocate, lock, or panic — that all belongs on the control thread.
//!
//! `src/effects/faust_dsp/*_gen.rs` is generated from `dsp/*.dsp` by
//! `dsp/regen.sh`; edit the `.dsp` source and regenerate, never the `.rs`.

#[cfg(feature = "native")]
pub mod audio;
#[cfg(feature = "native")]
pub mod cli_common;
#[cfg(feature = "native")]
pub mod config;
pub mod dsp;
pub mod effects;
#[cfg(feature = "native")]
pub mod error;
pub mod event;
#[cfg(feature = "native")]
pub mod offline;
pub mod orbit;
#[cfg(feature = "native")]
pub mod osc;
pub mod patch;
#[cfg(feature = "native")]
mod recorder;
pub mod sampling;
pub mod schedule;
#[cfg(feature = "soundfont")]
pub mod soundfont;
pub mod superpan;
#[cfg(feature = "native")]
pub mod telemetry;
#[cfg(feature = "native")]
pub mod time;
pub mod types;
pub mod voice;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[allow(clippy::large_enum_variant)]
pub enum AudioCmd {
    /// Pre-parsed event. Held inline (not boxed) so the audio thread does not
    /// deallocate a `Box` on receive. Spent events (after dispatch or when a
    /// scheduled event fires) are handed to the engine's reaper channel so
    /// their interior `String`/`Vec` fields are freed off the audio thread;
    /// only a full reaper falls back to dropping in place.
    DispatchEvent(event::Event),
    Hush,
    Panic,
}

use dsp::{fast_tanh_f32, ftz, init_envelope, DahdsrState};
use event::{Event, PatchParamValue};

use orbit::Orbit;

/// Re-export so downstream crates (e.g. `doux-sova`) can name the swap
/// type used by [`Engine::sample_index_handle`] without adding `arc-swap`
/// to their own `Cargo.toml`.
#[cfg(feature = "native")]
pub use arc_swap;
use patch::PatchRegistry;
#[cfg(feature = "native")]
use recorder::Recorder;
#[cfg(feature = "native")]
use sampling::RegistrySample;
use sampling::SampleEntry;
#[cfg(feature = "native")]
pub use sampling::SampleLoader;
#[cfg(feature = "native")]
pub use sampling::{SampleData, SampleRegistry};
#[cfg(not(feature = "native"))]
use sampling::{SampleInfo, SamplePool};
use schedule::Schedule;
use std::sync::Arc;
#[cfg(feature = "native")]
pub use telemetry::EngineMetrics;
#[cfg(feature = "native")]
use telemetry::ProfilePhase;
#[cfg(feature = "native")]
use types::DEFAULT_BUFFER_SIZE;
#[cfg(not(feature = "native"))]
use types::WASM_BUFFER_SIZE;
use types::{
    DspBlockSize, ModuleInfo, Source, CHANNELS, DEFAULT_DSP_BLOCK_SIZE, DEFAULT_MAX_VOICES,
    MAX_BLOCK, MAX_BUFFER_FRAMES, MAX_ORBITS,
};
use voice::modulation::{ModChain, ModCurve, ParamId};
use voice::{modulation, Voice, VoiceParams};

/// How `process_event` applies an event to its target voice.
///
/// `voice/N` is a stable identity tag; the presence of a sound decides
/// between updating the sounding voice and retriggering it.
#[derive(Clone, Copy, PartialEq)]
enum EventMode {
    /// Fresh or fully reset voice: defaults, then the event snapshot.
    New,
    /// Sounding voice, no sound named: retarget params only — envelope,
    /// gate, phase and sample position are untouched.
    Update,
    /// Sounding voice with a sound named: params stay sticky, envelopes
    /// re-fire from their current value.
    Retrigger,
}

/// All modules in the engine: sources, effects, filters, modulation.
///
/// Public surface consumed by sova's docs panel; keep stable.
pub fn all_modules() -> Vec<&'static ModuleInfo> {
    let mut modules: Vec<&'static ModuleInfo> =
        Source::all().iter().map(|s| &s.info().module).collect();
    modules.extend_from_slice(effects::ALL_MODULES);
    modules
}

#[cfg(feature = "soundfont")]
struct GmResolved {
    data: Arc<SampleData>,
    root_freq: f32,
    sr_ratio: f32,
    loop_start: f32,
    loop_end: f32,
    looping: bool,
    loop_until_release: bool,
    attenuation: f32,
    pan: f32,
    /// Initial filter cutoff in Hz; `None` when the zone leaves the filter open.
    filter_fc: Option<f32>,
    filter_q: f32,
    scale_tuning: f32,
    vib_rate: f32,
    vib_depth: f32,
    exclusive_class: u8,
    delay: f32,
    hold: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

/// Upper bound on interleaved output channels: superpan addresses at most
/// `MAX_SUPERPAN_NODES` stereo pairs, so wider devices gain nothing. Lets
/// per-channel master state live in fixed arrays (no audio-path allocation).
pub const MAX_OUTPUT_CHANNELS: usize = 2 * superpan::MAX_SUPERPAN_NODES;
/// Corner of the master DC-blocking one-pole high-pass.
const MASTER_DC_HZ: f32 = 10.0;
/// Ceiling on the master output level (~+6 dB). Boost is allowed because the
/// gain lands before the limiter's peak detector, so it cannot escape the
/// safety chain; the cap only stops a fat-fingered value from driving the
/// limiter into permanent gain reduction.
pub const MASTER_GAIN_MAX: f32 = 2.0;
/// Ceiling on the bass-mono crossover corner. Club practice puts it near
/// 100-120 Hz; past a few hundred the collapse is audible on the whole mix.
pub const BASS_MONO_MAX_HZ: f32 = 300.0;

/// The loudest sample the engine can emit, linear. The limiter holds the peak
/// envelope at `LIMITER_THRESHOLD` and the safety clip after it rounds that
/// down, so nothing above this reaches a host. Derived rather than written
/// down: a host that scales a meter to the real ceiling must not have to
/// restate the limiter threshold or the shape of the clip.
pub fn master_ceiling() -> f32 {
    soft_clip_sample(effects::LIMITER_THRESHOLD)
}

// Master safety clip, after the limiter: plain tanh. Identity slope at origin,
// monotonic, bounded by ±1. The limiter holds peaks near LIMITER_THRESHOLD,
// so program material passes this at near-unity slope; only attack edges the
// limiter's instant-attack envelope hasn't caught yet get rounded.
#[inline]
fn soft_clip_sample(input: f32) -> f32 {
    fast_tanh_f32(input)
}

/// Fill `gains[..n]` with orbit `oi`'s compressor gain, advancing its envelope
/// one sample per frame. Returns false when the compressor is disengaged, so
/// the caller can skip the multiply entirely.
///
/// Split out because both consumers of an orbit's audio need it: the
/// stereo-pair mix (Pass 1) and the room spread (Pass 3). A room-routed orbit
/// that skipped this would freeze its envelope mid-flight and resume from a
/// stale value when the room latch released.
///
/// Sidechain levels are staged through a stack scratch so reading another
/// orbit's post-FX bus does not alias the mutable borrow of this one; that
/// staging is also what makes `sc == oi` (self-compression) legal.
fn orbit_comp_gains(
    orbits: &mut [Orbit; MAX_ORBITS],
    oi: usize,
    isr: f32,
    n: usize,
    gains: &mut [f32; MAX_BLOCK],
) -> bool {
    let cp = orbits[oi].comp.params;
    if cp.amount <= 0.0 {
        return false;
    }
    // No `comporbit` means "this orbit", so a bare `comp` glues rather than
    // ducking from whatever orbit 0 is playing.
    let sc = cp_orbit_of(orbits, oi);
    let attack_coeff = (isr / cp.attack.max(0.0001)).min(1.0);
    let release_coeff = (isr / cp.release.max(0.0001)).min(1.0);
    let mut sc_lv = [0.0f32; MAX_BLOCK];
    for (slot, frame) in sc_lv.iter_mut().zip(orbits[sc].bus.iter()).take(n) {
        *slot = frame[0].abs().max(frame[1].abs());
    }
    let (thresh, exponent) = cp.gain_coeffs();
    let orbit = &mut orbits[oi];
    for (f, &sc_level) in sc_lv.iter().enumerate().take(n) {
        let env = orbit.comp.process(sc_level, attack_coeff, release_coeff);
        gains[f] = cp.gain_for(env, thresh, exponent);
    }
    true
}

/// The orbit whose bus feeds `oi`'s detector: its own when `comporbit` is unset.
#[inline]
fn cp_orbit_of(orbits: &[Orbit; MAX_ORBITS], oi: usize) -> usize {
    match orbits[oi].comp_orbit {
        Some(sc) => sc % MAX_ORBITS,
        None => oi,
    }
}

/// Construction-time configuration for [`Engine`]. Every field is set
/// once; runtime mutation goes through methods on the Engine itself.
#[derive(Clone)]
pub struct EngineConfig {
    /// Sample rate in Hz. Must match the audio device.
    pub sample_rate: f32,
    /// Number of interleaved output channels in `process_block`'s output slice.
    pub output_channels: usize,
    /// Maximum simultaneous voices (polyphony cap).
    pub max_voices: usize,
    /// Host audio callback size in samples (per channel). Sets pre-allocated
    /// output buffers. Distinct from `inner_block_size` — this is what the
    /// device hands us; the inner block is how finely we slice it for DSP.
    pub host_buffer_size: usize,
    /// Inner DSP block size in samples. Clamped to `[1, MAX_BLOCK]` at
    /// construction; finer-grained value yields lower latency for sample-rate
    /// scheduling at the cost of throughput. Always `<= host_buffer_size`.
    pub inner_block_size: usize,
    /// Caller-provided telemetry handle. Clone once before construction so
    /// the host can read metrics while the audio thread owns the engine.
    #[cfg(feature = "native")]
    pub metrics: Arc<EngineMetrics>,
    /// Reuse an existing sample registry (e.g. across device-loss recovery).
    /// `None` constructs a fresh one.
    #[cfg(feature = "native")]
    pub sample_registry: Option<Arc<SampleRegistry>>,
    /// Reuse an existing arf patch registry (same recovery pattern as
    /// `sample_registry`). `None` constructs a fresh one.
    pub patch_registry: Option<Arc<PatchRegistry>>,
}

impl EngineConfig {
    /// Native defaults, requires the platform-determined sample rate and
    /// output-channel count.
    #[cfg(feature = "native")]
    pub fn native(sample_rate: f32, output_channels: usize) -> Self {
        Self {
            sample_rate,
            output_channels,
            max_voices: DEFAULT_MAX_VOICES,
            host_buffer_size: DEFAULT_BUFFER_SIZE,
            inner_block_size: DEFAULT_DSP_BLOCK_SIZE,
            metrics: Arc::new(EngineMetrics::default()),
            sample_registry: None,
            patch_registry: None,
        }
    }

    /// WASM defaults (worklet-quantum buffer size).
    #[cfg(not(feature = "native"))]
    pub fn wasm(sample_rate: f32, output_channels: usize) -> Self {
        Self {
            sample_rate,
            output_channels,
            max_voices: DEFAULT_MAX_VOICES,
            host_buffer_size: WASM_BUFFER_SIZE,
            inner_block_size: DEFAULT_DSP_BLOCK_SIZE,
            patch_registry: None,
        }
    }
}

pub struct Engine {
    pub(crate) sr: f32,
    /// INVARIANT: held equal to `1.0 / sr`. No setter exists.
    pub(crate) isr: f32,
    pub(crate) max_voices: usize,
    pub(crate) voices: Vec<Voice>,
    pub(crate) active_voices: usize,
    /// Boxed: ~530KB inline would not fit alongside construction temporaries
    /// in the 1MB wasm32 shadow stack.
    pub(crate) orbits: Box<[Orbit; MAX_ORBITS]>,
    pub(crate) schedule: Schedule,
    pub(crate) time: f64,
    pub(crate) tick: u64,
    /// Earliest `tick` a NaN-latched patch Vm may be healed (swapped for a fresh
    /// pooled one). Bounds the off-RT reaper's reset/memset load to ~one heal per
    /// second engine-wide — a permanently-NaN 4 MiB-buffer patch would otherwise
    /// cost the reaper hundreds of MB/s. See the heal path in `gen_block`.
    next_heal_tick: u64,
    /// Transport tempo in beats per second, latched into every live patch's
    /// `BPS_LANE` once per chunk so arf graphs can read `bps`. Set from the
    /// host via [`Engine::set_tempo`]; defaults to 2.0 (120 BPM).
    tempo_bps: f32,
    pub(crate) output_channels: usize,
    /// Equal-power room-wet normalization `1/sqrt(output_channels)`, an
    /// engine-lifetime constant (output width is fixed at construction).
    room_gain: f32,
    pub(crate) host_buffer_size: usize,
    /// Inner DSP block size; sized scratch buffers guarantee `.get() ≤ MAX_BLOCK`.
    pub(crate) inner_block_size: DspBlockSize,
    pub(crate) output: Vec<f32>,
    /// Per-chunk, N-wide interleaved dry accumulator for `superpan` voices.
    /// Sized `MAX_BLOCK * output_channels`; summed into `output` at final mix.
    pub(crate) superpan_acc: Vec<f32>,
    /// True when `superpan_acc` may hold nonzero data. Cleared flag guarantees
    /// the whole buffer is zero, so idle chunks skip both clear and mix-in.
    pub(crate) superpan_acc_used: bool,
    /// Per-orbit compressor gain scratch, reused across orbits and chunks.
    /// Boxed and persistent rather than a per-`gen_block` stack array: only an
    /// engaged compressor writes it, and a stack array would memset 1 KB on
    /// every chunk (~1500/s) for engines that never use `comp` at all.
    comp_gains: Box<[f32; MAX_BLOCK]>,
    /// Master DC-blocker one-pole HP state, one slot per output channel.
    /// Persists across chunks — `gen_block` must never reset it.
    master_dc: [f32; MAX_OUTPUT_CHANNELS],
    /// One-pole coeff for the `MASTER_DC_HZ` high-pass (sr is fixed for the
    /// engine's lifetime, same invariant as `isr`).
    master_dc_coeff: f32,
    /// Master linked peak limiter (state persists across chunks).
    limiter: effects::Limiter,
    /// Master output level, applied in the final mix *before* peak detection so
    /// the limiter still protects a boosted master. `MASTER_GAIN_MAX` caps it.
    master_gain: f32,
    /// Previous chunk's master gain, for the per-sample ramp (de-zippers a live
    /// level move; identical to the send-level ramps on the orbit).
    prev_master_gain: f32,
    /// Bass-mono crossover corner in Hz; 0 disables the stage entirely.
    bass_mono_hz: f32,
    /// One-pole coeff for `bass_mono_hz`, recomputed only when the corner moves.
    bass_mono_coeff: f32,
    /// Bass-mono low-band state, one slot per output channel. Persists across
    /// chunks like `master_dc`.
    bass_mono_lp: [f32; MAX_OUTPUT_CHANNELS],
    #[cfg(not(feature = "native"))]
    pub(crate) sample_pool: SamplePool,
    #[cfg(not(feature = "native"))]
    pub(crate) samples: Vec<SampleInfo>,
    #[cfg(feature = "native")]
    pub(crate) sample_index: Arc<arc_swap::ArcSwap<Vec<SampleEntry>>>,
    #[cfg(not(feature = "native"))]
    pub(crate) sample_index: Vec<SampleEntry>,
    #[cfg(feature = "native")]
    pub(crate) sample_registry: Arc<SampleRegistry>,
    /// Installed arf patches (`s/<name>`), published from control threads,
    /// read lock-free at dispatch. The arf mirror of `sample_registry`.
    pub(crate) patch_registry: Arc<PatchRegistry>,
    #[cfg(feature = "native")]
    pub(crate) sample_loader: SampleLoader,
    #[cfg(feature = "native")]
    recorder: Recorder,
    #[cfg(feature = "native")]
    orbit_rec_bus: Vec<f32>,
    #[cfg(feature = "native")]
    pub(crate) metrics: Arc<EngineMetrics>,
    #[cfg(feature = "soundfont")]
    pub(crate) gm_bank: Arc<arc_swap::ArcSwapOption<soundfont::GmBank>>,
    pub(crate) input_channels: usize,
    voice_seed: u32,
    #[cfg(feature = "native")]
    load_gate: bool,
    #[cfg(feature = "native")]
    engine_start_unix_micros: u64,
    /// Spent events go here so their heap fields are freed by the reaper
    /// thread, not the audio thread. `None` (spawn failure) or a full queue
    /// degrades to dropping in place.
    #[cfg(feature = "native")]
    event_reaper: Option<crossbeam_channel::Sender<Event>>,
}

#[cfg(feature = "native")]
fn now_unix_micros() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// Steal preference class: releasing or dead voices first, established
/// (post-attack) voices next, still-attacking voices last — a burst at the
/// ceiling must not eat its own newest notes while a release tail survives.
fn steal_class(v: &Voice) -> u8 {
    match v.dahdsr.state() {
        DahdsrState::Off | DahdsrState::Release => 0,
        DahdsrState::Decay | DahdsrState::Sustain => 1,
        DahdsrState::Delay | DahdsrState::Attack | DahdsrState::Hold => 2,
    }
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        assert!(
            config.output_channels <= MAX_OUTPUT_CHANNELS,
            "output_channels exceeds MAX_OUTPUT_CHANNELS — master per-channel state is fixed-size"
        );
        dsp::fft::init_twiddles();
        // Eagerly init stretch's LazyLock tables off the audio thread so the
        // first stretched-sample play does not pay the init cost on RT.
        // Stretch lives under `native` only — WASM has no time-stretch path.
        #[cfg(feature = "native")]
        sampling::init_stretch_tables();

        // Built through a Vec so each Orbit (~66KB) is the only stack temporary;
        // the full array never exists outside the heap.
        let orbits: Vec<Orbit> = (0..MAX_ORBITS)
            .map(|i| Orbit::new(config.sample_rate, i))
            .collect();
        let Ok(orbits) = Box::<[Orbit; MAX_ORBITS]>::try_from(orbits.into_boxed_slice()) else {
            unreachable!("collected exactly MAX_ORBITS orbits");
        };

        #[cfg(feature = "native")]
        let (sample_registry, sample_loader) = {
            let registry = config
                .sample_registry
                .unwrap_or_else(|| Arc::new(SampleRegistry::new()));
            let loader = SampleLoader::new(Arc::clone(&registry));
            (registry, loader)
        };

        #[cfg(feature = "native")]
        let sample_index: Arc<arc_swap::ArcSwap<Vec<SampleEntry>>> =
            Arc::new(arc_swap::ArcSwap::from_pointee(Vec::new()));
        #[cfg(feature = "native")]
        let recorder = Recorder::new(
            config.sample_rate,
            Arc::clone(&config.metrics),
            Arc::clone(&sample_registry),
            Arc::clone(&sample_index),
        );

        // Reaper: spent events cross back over this channel so their heap
        // fields are freed here, not in the audio callback. Sized to the
        // schedule depth; the thread exits when the engine (sender) drops.
        #[cfg(feature = "native")]
        let event_reaper = {
            let (tx, rx) = crossbeam_channel::bounded::<Event>(types::MAX_EVENTS);
            std::thread::Builder::new()
                .name("doux-event-reaper".into())
                .spawn(move || while rx.recv().is_ok() {})
                .ok()
                .map(|_| tx)
        };

        Self {
            sr: config.sample_rate,
            isr: 1.0 / config.sample_rate,
            max_voices: config.max_voices,
            voices: (0..config.max_voices).map(|_| Voice::default()).collect(),
            active_voices: 0,
            orbits,
            schedule: Schedule::new(),
            time: 0.0,
            tick: 0,
            next_heal_tick: 0,
            tempo_bps: 2.0,
            output_channels: config.output_channels,
            room_gain: 1.0 / (config.output_channels as f32).sqrt(),
            host_buffer_size: config.host_buffer_size,
            inner_block_size: DspBlockSize::new(config.inner_block_size),
            output: vec![0.0; config.host_buffer_size * config.output_channels],
            superpan_acc: vec![0.0; MAX_BLOCK * config.output_channels],
            superpan_acc_used: false,
            comp_gains: Box::new([1.0; MAX_BLOCK]),
            master_dc: [0.0; MAX_OUTPUT_CHANNELS],
            // Bilinear one-pole coeff: w = PI * f / sr, coeff = 2w / (1 + 2w).
            master_dc_coeff: {
                let w = std::f32::consts::PI * MASTER_DC_HZ / config.sample_rate;
                (2.0 * w) / (1.0 + 2.0 * w)
            },
            limiter: effects::Limiter::default(),
            master_gain: 1.0,
            prev_master_gain: 1.0,
            bass_mono_hz: 0.0,
            bass_mono_coeff: 0.0,
            bass_mono_lp: [0.0; MAX_OUTPUT_CHANNELS],
            #[cfg(not(feature = "native"))]
            sample_pool: SamplePool::new(),
            #[cfg(not(feature = "native"))]
            samples: Vec::with_capacity(256),
            #[cfg(feature = "native")]
            sample_index,
            #[cfg(not(feature = "native"))]
            sample_index: Vec::new(),
            #[cfg(feature = "native")]
            sample_registry,
            patch_registry: config
                .patch_registry
                .unwrap_or_else(|| Arc::new(PatchRegistry::with_polyphony(config.max_voices))),
            #[cfg(feature = "native")]
            sample_loader,
            #[cfg(feature = "native")]
            recorder,
            #[cfg(feature = "native")]
            orbit_rec_bus: vec![0.0; MAX_ORBITS * MAX_BUFFER_FRAMES * CHANNELS],
            #[cfg(feature = "native")]
            metrics: config.metrics,
            #[cfg(feature = "soundfont")]
            gm_bank: Arc::new(arc_swap::ArcSwapOption::const_empty()),
            input_channels: 2,
            voice_seed: 123456789,
            #[cfg(feature = "native")]
            load_gate: false,
            #[cfg(feature = "native")]
            engine_start_unix_micros: now_unix_micros(),
            #[cfg(feature = "native")]
            event_reaper,
        }
    }

    /// Hand a spent event to the reaper so its `String`/`Vec` fields are
    /// freed off the audio thread. Falls back to dropping in place when the
    /// reaper is full or absent (wasm) — the pre-reaper behavior.
    #[inline]
    fn retire_event(&self, event: Event) {
        #[cfg(feature = "native")]
        if let Some(tx) = &self.event_reaper {
            // On Full/Disconnected the error carries the event back and it
            // drops here.
            let _ = tx.try_send(event);
            return;
        }
        drop(event);
    }

    /// Empty the schedule through the reaper (`Schedule::clear` would free
    /// every queued event's heap fields on the audio thread).
    fn drain_schedule(&mut self) {
        while let Some(ev) = self.schedule.pop_front() {
            self.retire_event(ev);
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sr
    }

    pub fn output_channels(&self) -> usize {
        self.output_channels
    }

    pub fn host_buffer_size(&self) -> usize {
        self.host_buffer_size
    }

    pub fn inner_block_size(&self) -> usize {
        self.inner_block_size.get()
    }

    pub fn max_voices(&self) -> usize {
        self.max_voices
    }

    pub fn active_voices(&self) -> usize {
        self.active_voices
    }

    /// Snapshot of the sample index.
    ///
    /// Native: returns an `arc_swap::Guard` that derefs through
    /// `Arc<Vec<SampleEntry>>` to `Vec<SampleEntry>` / `[SampleEntry]`. The
    /// load is a single atomic op — RT-safe.
    #[cfg(feature = "native")]
    pub fn sample_index(&self) -> arc_swap::Guard<Arc<Vec<SampleEntry>>> {
        self.sample_index.load()
    }

    #[cfg(not(feature = "native"))]
    pub fn sample_index(&self) -> &[SampleEntry] {
        &self.sample_index
    }

    /// Handle to the swappable sample-index slot. Worker threads clone this
    /// to publish a new index without going through the RT thread.
    #[cfg(feature = "native")]
    pub fn sample_index_handle(&self) -> Arc<arc_swap::ArcSwap<Vec<SampleEntry>>> {
        Arc::clone(&self.sample_index)
    }

    /// Atomically replaces the sample index. `&self` — interior mutation via
    /// `ArcSwap`. The previous `Vec` is dropped on whichever thread drops the
    /// last `Arc`; for the worker-driven flow, that's the worker.
    #[cfg(feature = "native")]
    pub fn set_sample_index(&self, index: Vec<SampleEntry>) {
        self.sample_index.store(Arc::new(index));
    }

    #[cfg(not(feature = "native"))]
    pub fn set_sample_index(&mut self, index: Vec<SampleEntry>) {
        self.sample_index = index;
    }

    /// Appends `entries` to the sample index. Clones the current Vec on the
    /// caller's thread (off the RT path).
    #[cfg(feature = "native")]
    pub fn extend_sample_index<I: IntoIterator<Item = SampleEntry>>(&self, entries: I) {
        let mut new_index = (*self.sample_index.load_full()).clone();
        new_index.extend(entries);
        self.sample_index.store(Arc::new(new_index));
    }

    #[cfg(not(feature = "native"))]
    pub fn extend_sample_index<I: IntoIterator<Item = SampleEntry>>(&mut self, entries: I) {
        self.sample_index.extend(entries);
    }

    pub fn set_input_channels(&mut self, n: usize) {
        self.input_channels = n;
    }

    #[cfg(feature = "native")]
    pub fn metrics(&self) -> &Arc<EngineMetrics> {
        &self.metrics
    }

    #[cfg(feature = "native")]
    pub fn sample_registry(&self) -> &Arc<SampleRegistry> {
        &self.sample_registry
    }

    /// Shared handle to the arf patch registry. Clone it before the Engine
    /// moves onto the audio thread; installs published through it are picked
    /// up by the next `s/<name>` event.
    pub fn patch_registry(&self) -> &Arc<PatchRegistry> {
        &self.patch_registry
    }

    /// Shared handle to the GM-bank slot, so an off-RT worker can publish a
    /// decoded soundfont via `store` (mirrors `sample_index_handle`).
    #[cfg(feature = "soundfont")]
    pub fn gm_bank_handle(&self) -> Arc<arc_swap::ArcSwapOption<soundfont::GmBank>> {
        Arc::clone(&self.gm_bank)
    }

    #[cfg(feature = "native")]
    pub fn time_anchor(&self) -> time::TimeAnchor {
        time::TimeAnchor {
            start_unix_micros: self.engine_start_unix_micros,
            sample_rate: self.sr,
        }
    }

    #[cfg(feature = "soundfont")]
    pub fn load_soundfont(&mut self, path: &std::path::Path) -> Result<(), String> {
        // The bank owns its sample PCM (Arc<SampleData>), so publishing it is a
        // single atomic store — the RT thread never observes a zone whose sample
        // isn't present, and nothing lands in the shared sample registry.
        let bank = soundfont::load_sf2(path, self.sr)?;
        self.gm_bank.store(Some(Arc::new(bank)));
        Ok(())
    }

    #[cfg(feature = "soundfont")]
    pub fn gm_bank(&self) -> Option<Arc<soundfont::GmBank>> {
        self.gm_bank.load_full()
    }

    #[cfg(feature = "soundfont")]
    pub fn take_gm_bank(&self) -> Option<Arc<soundfont::GmBank>> {
        self.gm_bank.swap(None)
    }

    #[cfg(feature = "soundfont")]
    pub fn set_gm_bank(&self, bank: Arc<soundfont::GmBank>) {
        self.gm_bank.store(Some(bank));
    }

    #[cfg(not(feature = "native"))]
    pub fn load_sample(&mut self, samples: &[f32], channels: u8, freq: f32) -> Option<usize> {
        // JS may hand a NaN/0 base freq, which feeds pitch-ratio math downstream.
        let freq = if freq.is_finite() && freq > 0.0 {
            freq
        } else {
            261.626
        };
        let info = self.sample_pool.add(samples, channels, freq)?;
        let idx = self.samples.len();
        self.samples.push(info);
        Some(idx)
    }

    /// Look up sample folder/n (e.g., "wave_tek/3"). `n` wraps via modulo over the folder count.
    /// Walks the index twice (count, then find) — but each walk is O(n) and shared by all callers.
    ///
    /// Returns a cloned [`SampleEntry`] (two `Arc` clones) so the caller does
    /// not need to keep the [`arc_swap::Guard`] alive.
    #[cfg(feature = "native")]
    fn lookup_sample_entry(&self, name: &str, n: usize) -> Option<SampleEntry> {
        let name_bytes = name.as_bytes();
        let name_len = name.len();
        let matches = |e: &SampleEntry| {
            e.name.len() > name_len
                && e.name.as_bytes()[name_len] == b'/'
                && e.name.as_bytes().starts_with(name_bytes)
        };
        let index = self.sample_index.load();
        let count = index.iter().filter(|e| matches(e)).count();
        if count == 0 {
            return None;
        }
        let wrapped_n = n % count;
        index
            .iter()
            .find(|e| matches(e) && e.name[name_len + 1..].parse::<usize>().ok() == Some(wrapped_n))
            .cloned()
    }

    /// Try to get a sample from the registry, or request background loading.
    #[cfg(feature = "native")]
    fn get_registry_sample(&mut self, name: &str, n: usize) -> Option<(Arc<str>, Arc<SampleData>)> {
        let entry = self.lookup_sample_entry(name, n)?;
        let sample_name = entry.name;
        let path = entry.path;

        if let Some(data) = self.sample_registry.get(sample_name.as_ref()) {
            if data.frame_count < data.total_frames {
                self.sample_loader
                    .request(Arc::clone(&sample_name), path, self.sr);
            }
            return Some((sample_name, data));
        }

        self.sample_loader.request(sample_name, path, self.sr);
        None
    }

    /// Resolve a GM soundfont zone for a note-on. RT-safe: program/bank
    /// resolution is allocation-free, the bank lookup is a binary search plus a
    /// short scan, and the sample is a cheap `Arc` clone owned by the bank.
    #[cfg(feature = "soundfont")]
    fn resolve_gm(&self, event: &Event) -> Option<GmResolved> {
        let sound_str = event.sound.as_ref()?;
        let suffix = sound_str.strip_prefix("gm")?;
        // Program selector. The inline single-token form bakes the preset into
        // the sound (`gmpiano`); the param form leaves the sound bare `gm` and
        // carries the preset in `n` (`gm snd piano n`). Suffix wins when present,
        // else fall back to `n`, defaulting to program 0 (piano). Allocation-free.
        let selector = if suffix.is_empty() {
            event.n.as_deref().unwrap_or("0")
        } else {
            suffix
        };
        let (program, bank) = soundfont::resolve_gm_program(selector)?;

        let note = event
            .freq
            .map(|f| (types::freq2midi(f).round() as i32).clamp(0, 127) as u8)
            .unwrap_or(60);
        let vel = (event.velocity.unwrap_or(1.0) * 127.0).clamp(1.0, 127.0) as u8;

        let bank_guard = self.gm_bank.load();
        let bank_ref = bank_guard.as_ref()?;
        let zone = bank_ref.find(program, bank, note, vel)?;

        // SF2 default modulator #1: note-on velocity → amplitude (concave,
        // FluidSynth-matched). The zone attenuation already folds in the
        // resonance −Q/2 dB makeup.
        let vel_gain = soundfont::cb_to_linear_gain(soundfont::velocity_to_attenuation_cb(vel));
        let attenuation = zone.attenuation * vel_gain;

        // Initial filter cutoff: cents → Hz; the ~19912 Hz "open" default leaves
        // the lowpass bypassed.
        let fc_hz = 8.176 * 2.0_f32.powf(zone.filter_fc_cents / 1200.0);
        let filter_fc = if fc_hz < 19500.0 { Some(fc_hz) } else { None };

        Some(GmResolved {
            data: zone.data,
            root_freq: zone.root_freq,
            sr_ratio: zone.sr_ratio,
            loop_start: zone.loop_start,
            loop_end: zone.loop_end,
            looping: zone.looping,
            loop_until_release: zone.loop_until_release,
            attenuation,
            pan: zone.pan,
            filter_fc,
            filter_q: zone.filter_q,
            scale_tuning: zone.scale_tuning,
            vib_rate: zone.vib_rate,
            vib_depth: zone.vib_depth,
            exclusive_class: zone.exclusive_class,
            delay: zone.delay,
            hold: zone.hold,
            attack: zone.attack,
            decay: zone.decay,
            sustain: zone.sustain,
            release: zone.release,
        })
    }

    /// Get a loaded sample index (WASM only - uses legacy pool)
    #[cfg(not(feature = "native"))]
    fn get_or_load_sample(&mut self, name: &str, _n: usize) -> Option<usize> {
        // For WASM, treat `name` as numeric index if sample_index is empty
        if self.sample_index.is_empty() {
            let idx: usize = name.parse().ok()?;
            if idx < self.samples.len() {
                return Some(idx);
            }
        }
        None
    }

    /// Parse and dispatch — only call this off the RT thread.
    pub fn evaluate(&mut self, input: &str) -> Option<usize> {
        let event = Event::parse(input, self.sr);
        self.dispatch_event(event)
    }

    /// Dispatch a pre-parsed event.
    ///
    /// All paths are RT-safe: `play` reuses pre-owned metadata and only clones
    /// `Arc` handles on the callback path; `rec` hands the captured buffer to
    /// a background worker for off-RT finalize.
    pub fn dispatch_event(&mut self, event: Event) -> Option<usize> {
        let cmd = event.cmd.as_deref().unwrap_or("play");

        match cmd {
            "play" => self.play_event(event),
            #[cfg(feature = "native")]
            "rec" => {
                self.handle_rec(event);
                None
            }
            "hush" => {
                self.hush();
                self.retire_event(event);
                None
            }
            "panic" => {
                self.panic();
                self.retire_event(event);
                None
            }
            "reset" => {
                self.panic();
                self.drain_schedule();
                self.time = 0.0;
                self.tick = 0;
                self.retire_event(event);
                None
            }
            "release" => {
                if let Some(tag) = event.voice {
                    for i in 0..self.active_voices {
                        if self.voices[i].tag == Some(tag) {
                            self.voices[i].force_release();
                        }
                    }
                }
                self.retire_event(event);
                None
            }
            "hush_endless" => {
                for i in 0..self.active_voices {
                    if self.voices[i].params.gate == 0.0 {
                        self.voices[i].force_release();
                    }
                }
                self.retire_event(event);
                None
            }
            "reset_time" => {
                self.time = 0.0;
                self.tick = 0;
                self.retire_event(event);
                None
            }
            "reset_schedule" => {
                self.drain_schedule();
                self.retire_event(event);
                None
            }
            _ => {
                self.retire_event(event);
                None
            }
        }
    }

    fn play_event(&mut self, mut event: Event) -> Option<usize> {
        if let Some(delta) = event.delta {
            let base = event.tick.unwrap_or(self.tick) as i64;
            event.tick = Some((base + delta).max(0) as u64);
            event.delta = None;
        }
        if event.tick.is_some() {
            if let Some(rejected) = self.schedule.push(event) {
                self.retire_event(rejected);
            }
            return None;
        }
        let voice = self.process_event(&event);
        self.retire_event(event);
        voice
    }

    /// Dispatch a `rec` event on the audio thread.
    ///
    /// `rec_stop` stops the active recording; otherwise a named `rec`/`dub`
    /// starts one. The RT side only flips state and pushes samples into the
    /// capture ring — finalize (`SampleData`, `SampleRegistry::insert`,
    /// `sample_index` update) and overdub mixing run on the writer thread.
    /// A nameless start is ignored (the Forth verbs always supply a name).
    #[cfg(feature = "native")]
    fn handle_rec(&mut self, mut event: Event) {
        if event.rec_stop.unwrap_or(false) {
            self.recorder.stop();
        } else if let Some(name) = event.sound.take() {
            self.recorder
                .start(name, event.overdub.unwrap_or(false), event.orbit);
        }
        self.retire_event(event);
    }

    pub fn play(&mut self, params: VoiceParams) -> Option<usize> {
        #[cfg(feature = "native")]
        if self.load_gate {
            return None;
        }
        if self.active_voices >= self.max_voices {
            return None;
        }
        let i = self.active_voices;
        if let Some(old) = self.voices[i].patch.take() {
            self.patch_registry.retire(old);
        }
        if let Some(old) = self.voices[i].fx_patch.take() {
            self.patch_registry.retire(old);
        }
        self.voices[i].reset();
        self.voices[i].seed = self.voice_seed;
        self.voice_seed = modulation::lcg(self.voice_seed);
        self.voices[i].params = params;
        self.voices[i].sr = self.sr;
        self.voices[i].sync_source_state();
        self.voices[i].ensure_effects();
        self.active_voices += 1;
        Some(i)
    }

    /// The steal victim when a `New` event arrives at the polyphony ceiling:
    /// a lexicographic minimum over `(steal_class(v), dahdsr.current_val)`.
    /// The class rank (releasing/dead, then established, then still-attacking)
    /// dominates so a burst at the ceiling never eats its own newest notes while
    /// a release tail survives; envelope value breaks ties within a class. Only
    /// called when `active_voices >= max_voices ≥ 1`, so the scan is non-empty
    /// and slot 0 is a valid default.
    fn steal_voice_slot(&self) -> usize {
        let mut min_idx = 0;
        let mut min_class = u8::MAX;
        let mut min_val = f32::MAX;
        for i in 0..self.active_voices {
            let class = steal_class(&self.voices[i]);
            let val = self.voices[i].dahdsr.current_val;
            if class < min_class || (class == min_class && val < min_val) {
                min_class = class;
                min_val = val;
                min_idx = i;
            }
        }
        min_idx
    }

    /// Process an event, handling voice selection like dough.c's process_engine_event()
    fn process_event(&mut self, event: &Event) -> Option<usize> {
        // Cut group: reuse first matching voice, hard_cut any extras
        let mut cut_reuse: Option<usize> = None;
        if let Some(cut) = event.cut {
            for i in 0..self.active_voices {
                if self.voices[i].params.cut == Some(cut) {
                    if cut_reuse.is_none() {
                        cut_reuse = Some(i);
                    } else {
                        self.voices[i].hard_cut();
                    }
                }
            }
        }

        // If sound is specified but doesn't resolve to anything, check availability
        // Skip this check if WebSample data is already present (WASM with JS-loaded sample)
        let has_web_sample = event.file_pcm.is_some() && event.file_frames.is_some();
        if let Some(ref sound_str) = event.sound {
            if !has_web_sample && sound_str.parse::<Source>().is_err() {
                if let Some(entry) = self.patch_registry.get(sound_str).filter(|e| e.is_source()) {
                    // A bare name resolves to a source patch first, then falls
                    // to a sample folder below (the `else`). An effect-role
                    // patch of this name is filtered out here and falls through
                    // too. A dry Vm pool drops the event, like the sample-miss
                    // drop below. The pool probe is reliable: dispatch and voice
                    // death both run on this thread, so nothing pops between
                    // here and the attach. A dry pool still admits a retrigger
                    // of a tagged voice already holding this patch — it reuses
                    // its Vm, not the pool's — but only when the event keeps the
                    // voice (a reset or a cut group takes the New path, which
                    // retires the Vm and would then find the pool dry and play
                    // silence).
                    if !entry.has_vm() {
                        let retargets_holder = !event.reset.unwrap_or(false)
                            && event.cut.is_none()
                            && event.voice.is_some_and(|tag| {
                                (0..self.active_voices).any(|i| {
                                    self.voices[i].tag == Some(tag)
                                        && self.voices[i]
                                            .patch
                                            .as_ref()
                                            .is_some_and(|p| Arc::ptr_eq(&p.entry, &entry))
                                })
                            });
                        if !retargets_holder {
                            return None;
                        }
                    }
                } else {
                    let effective_name = event.effective_name.as_deref().unwrap_or(sound_str);
                    #[cfg(feature = "native")]
                    {
                        let n = event.n_as_index();
                        self.get_registry_sample(effective_name, n)?;
                    }
                    #[cfg(not(feature = "native"))]
                    {
                        let n = event.n_as_index();
                        self.get_or_load_sample(effective_name, n)?;
                    }
                }
            }
        }

        // Voice insert availability gate, mirroring the source-patch sound gate
        // above: a registry miss, a source-role patch, or a dry Vm pool
        // drops the event — except when it retriggers a tagged voice already
        // holding this insert (which reuses its Vm, not the pool's).
        if let Some(fx_name) = event.fx.as_deref().filter(|n| *n != "off") {
            let entry = self.patch_registry.get(fx_name)?;
            if !entry.is_effect() {
                return None;
            }
            if !entry.has_vm() {
                let retargets_holder = !event.reset.unwrap_or(false)
                    && event.cut.is_none()
                    && event.voice.is_some_and(|tag| {
                        (0..self.active_voices).any(|i| {
                            self.voices[i].tag == Some(tag)
                                && self.voices[i]
                                    .fx_patch
                                    .as_ref()
                                    .is_some_and(|p| Arc::ptr_eq(&p.entry, &entry))
                        })
                    });
                if !retargets_holder {
                    return None;
                }
            }
        }

        // `voice/N` is an identity tag: scan the sounding voices for it.
        let tagged = event
            .voice
            .and_then(|tag| (0..self.active_voices).find(|&i| self.voices[i].tag == Some(tag)));
        let has_sound = event.sound.is_some() || has_web_sample;

        // Set when a New event steals an existing slot at the polyphony ceiling;
        // the New block below then carries the victim's envelope for a click-free
        // takeover (like a cut-group reuse).
        let mut stole_slot = false;
        let (voice_idx, mode) = if let Some(reuse_idx) = cut_reuse {
            (reuse_idx, EventMode::New)
        } else if let Some(idx) = tagged {
            if event.reset.unwrap_or(false) {
                (idx, EventMode::New)
            } else if has_sound {
                (idx, EventMode::Retrigger)
            } else {
                (idx, EventMode::Update)
            }
        } else if event.voice.is_some() && !has_sound {
            // Addressing a voice that isn't sounding without naming a
            // sound: drop — a tweak must not spawn a default voice.
            return None;
        } else {
            // Allocate new — or, at the polyphony ceiling, steal the quietest
            // voice so the newest note always sounds. Native still honors
            // `load_gate` (an engine reset in progress drops the event).
            #[cfg(feature = "native")]
            if self.load_gate {
                return None;
            }
            if self.active_voices >= self.max_voices {
                stole_slot = true;
                (self.steal_voice_slot(), EventMode::New)
            } else {
                let i = self.active_voices;
                self.active_voices += 1;
                (i, EventMode::New)
            }
        };

        if mode == EventMode::New {
            let old_env = if cut_reuse.is_some() || stole_slot {
                self.voices[voice_idx].dahdsr.current_val
            } else {
                0.0
            };
            // A reused slot may still hold arf Vms (source and/or insert):
            // send them back to their pools before `reset` — dropping them
            // here would free on RT.
            if let Some(old) = self.voices[voice_idx].patch.take() {
                self.patch_registry.retire(old);
            }
            if let Some(old) = self.voices[voice_idx].fx_patch.take() {
                self.patch_registry.retire(old);
            }
            self.voices[voice_idx].reset();
            self.voices[voice_idx].dahdsr.current_val = old_env;
            self.voices[voice_idx].seed = self.voice_seed;
            self.voice_seed = modulation::lcg(self.voice_seed);
            self.voices[voice_idx].sr = self.sr;
        }
        self.voices[voice_idx].tag = event.voice;

        // Update voice params (only the ones explicitly set in event)
        self.update_voice_params(voice_idx, event, mode);

        // SF2 exclusiveClass: a new note in a non-zero class silences any other
        // voice sounding in the same class on the same orbit (hi-hat / drum choke).
        #[cfg(feature = "soundfont")]
        {
            let class = self.voices[voice_idx].exclusive_class;
            if class != 0 {
                let orbit = self.voices[voice_idx].params.orbit;
                for j in 0..self.active_voices {
                    if j != voice_idx
                        && self.voices[j].exclusive_class == class
                        && self.voices[j].params.orbit == orbit
                    {
                        self.voices[j].force_release();
                    }
                }
            }
        }

        if mode == EventMode::Retrigger {
            self.voices[voice_idx].retrigger();
        }
        self.voices[voice_idx].ensure_effects();

        Some(voice_idx)
    }

    /// Update voice params - only updates fields that are explicitly set in the event
    fn update_voice_params(&mut self, idx: usize, event: &Event, mode: EventMode) {
        macro_rules! copy_opt {
            ($src:expr, $dst:expr, $($field:ident),+ $(,)?) => {
                $(if let Some(val) = $src.$field { $dst.$field = val; })+
            };
        }
        macro_rules! copy_opt_some {
            ($src:expr, $dst:expr, $($field:ident),+ $(,)?) => {
                $(if let Some(val) = $src.$field { $dst.$field = Some(val); })+
            };
        }
        // Resolve sound/sample first (before borrowing voice). A bare name
        // resolves against the patch registry (a source-role patch) when it is
        // neither a builtin Source nor web-sample PCM; a builtin Source wins
        // over a same-named patch, and a miss falls back to a sample folder.
        // JS-supplied web-sample PCM wins over the sound name (same precedence
        // as the dispatch gate) — resolving the patch anyway would check a Vm
        // out of the pool only for the web-sample block to orphan it below.
        let has_web_sample = event.file_pcm.is_some() && event.file_frames.is_some();
        let patch_entry = event
            .sound
            .as_deref()
            .filter(|_| !has_web_sample)
            .filter(|s| s.parse::<Source>().is_err()) // a builtin source wins over a same-named patch
            .and_then(|name| self.patch_registry.get(name))
            .filter(|e| e.is_source());
        let fx_entry = event
            .fx
            .as_deref()
            .filter(|n| *n != "off")
            .and_then(|name| self.patch_registry.get(name))
            .filter(|e| e.is_effect());

        #[cfg(feature = "native")]
        let (registry_sample_data, registry_sample_data_b, sample_blend) =
            if let Some(ref sound_str) = event.sound {
                if sound_str.parse::<Source>().is_ok() || patch_entry.is_some() {
                    (None, None, 0.0f32)
                } else {
                    let effective_name = event.effective_name.as_deref().unwrap_or(sound_str);
                    let n_float = event.n_as_float();
                    let n_floor = n_float.floor() as usize;
                    let blend = n_float.fract();
                    let a = self.get_registry_sample(effective_name, n_floor);
                    let b = if blend > 0.0 {
                        self.get_registry_sample(effective_name, n_floor + 1)
                    } else {
                        None
                    };
                    (a, b, blend)
                }
            } else {
                (None, None, 0.0)
            };

        let parsed_source = if let Some(ref sound_str) = event.sound {
            sound_str.parse::<Source>().ok()
        } else {
            None
        };

        // Resolve GM soundfont zone (before borrowing voice)
        #[cfg(feature = "soundfont")]
        let gm_resolved: Option<GmResolved> = if parsed_source == Some(Source::Gm) {
            self.resolve_gm(event)
        } else {
            None
        };

        #[cfg(not(feature = "native"))]
        let loaded_sample = if let Some(ref sound_str) = event.sound {
            if sound_str.parse::<Source>().is_err() && patch_entry.is_none() {
                let effective_name = event.effective_name.as_deref().unwrap_or(sound_str);
                let n = event.n_as_index();
                self.get_or_load_sample(effective_name, n)
            } else {
                None
            }
        } else {
            None
        };

        // --- Orbit FX state (Tidal-style sticky bus) ---
        // Resolve the target orbit from the event override or the voice's
        // current routing, then write any event-supplied FX params into the
        // orbit. The orbit is the source of truth for all FX state.
        let target_orbit = event
            .orbit
            .map(|o| o % MAX_ORBITS)
            .unwrap_or_else(|| self.voices[idx].params.orbit % MAX_ORBITS);
        {
            let orbit = &mut self.orbits[target_orbit];
            macro_rules! set {
                ($evt:ident, $dst:expr) => {
                    if let Some(x) = event.$evt {
                        $dst = x;
                    }
                };
            }
            macro_rules! set_pos {
                ($evt:ident, $dst:expr) => {
                    if let Some(x) = event.$evt {
                        $dst = x.max(0.0);
                    }
                };
            }
            // For params whose range is enforced rather than merely documented:
            // routes the static write through the same `write_param` the
            // ModChain path uses, so one clamp governs both.
            macro_rules! set_clamped {
                ($evt:ident, $id:ident) => {
                    if let Some(x) = event.$evt {
                        orbit.write_param(orbit::OrbitParamId::$id, x);
                    }
                };
            }
            // Statics displace any active ModChain on the same orbit param,
            // mirroring the voice-param semantics below.
            for &id in &event.orbit_static_ids {
                orbit.clear_mod(id);
            }
            set_pos!(delay, orbit.delay_level);
            set_pos!(verb, orbit.verb_level);
            set_pos!(comb, orbit.comb_level);
            set_pos!(feedback, orbit.fb_level);
            // Params with a real range go through `write_param` so the static
            // and ModChain paths cannot disagree about the clamp. `set!` writes
            // the field raw, which left `read_param` (and any chain that starts
            // from it) seeing a value the modulation path would have rejected.
            set_clamped!(comp, Comp);
            set!(delaytime, orbit.delay_params.time);
            set!(delayfeedback, orbit.delay_params.feedback);
            set!(delaytype, orbit.delay_params.delay_type);
            set!(verbtype, orbit.reverb_params.verb_type);
            set!(verbdecay, orbit.reverb_params.decay);
            set!(verbdamp, orbit.reverb_params.damp);
            set!(verbpredelay, orbit.reverb_params.predelay);
            set!(verbdiff, orbit.reverb_params.diff);
            set!(verbsize, orbit.reverb_params.size);
            set!(verbprelow, orbit.reverb_params.prelow);
            set!(verbprehigh, orbit.reverb_params.prehigh);
            set!(verblowcut, orbit.reverb_params.lowcut);
            set!(verbhighcut, orbit.reverb_params.highcut);
            set!(verblowgain, orbit.reverb_params.lowgain);
            set!(verbhighgain, orbit.reverb_params.highgain);
            set!(verbchorus, orbit.reverb_params.chorus);
            set!(verbchorusfreq, orbit.reverb_params.chorus_freq);
            set!(combfreq, orbit.comb_params.freq);
            set!(combfeedback, orbit.comb_params.feedback);
            set!(combdamp, orbit.comb_params.damp);
            set!(fbtime, orbit.fb_params.time_ms);
            set!(fbdamp, orbit.fb_params.damp);
            set!(fbcross, orbit.fb_params.cross);
            set!(compattack, orbit.comp.params.attack);
            set!(comprelease, orbit.comp.params.release);
            set_clamped!(compthresh, CompThresh);
            set_clamped!(compratio, CompRatio);
            set!(comporbit, orbit.comp_orbit);
            set_clamped!(patchlevel, PatchLevel);
            // Orbit arf patch (sticky): `patch/off` returns the Vm home; a
            // new name swaps in a Vm from that entry's pool. Same-entry
            // re-sends are no-ops so cagire's per-cycle param loops don't
            // churn the pool; a dry pool keeps the current patch. Names that
            // miss the registry or resolve to a source patch are ignored —
            // sticky-param semantics, the orbit keeps what it has.
            if let Some(ref name) = event.patch {
                if name == "off" {
                    if let Some(old) = orbit.patch.take() {
                        self.patch_registry.retire(old);
                    }
                } else if let Some(entry) = self.patch_registry.get(name).filter(|e| e.is_effect())
                {
                    let same = orbit
                        .patch
                        .as_ref()
                        .is_some_and(|p| Arc::ptr_eq(&p.entry, &entry));
                    if !same {
                        if let Some(vm) = entry.take_vm() {
                            if let Some(old) =
                                orbit.patch.replace(patch::VoicePatch::new(entry, vm))
                            {
                                self.patch_registry.retire(old);
                            }
                        }
                    }
                }
            }
            // Inline mods install last (an event carrying both a static and a
            // chain on the same param keeps the chain). Envelope chains
            // trigger on install; the event gate sets their release point
            // (0.0 = hold at sustain).
            let mod_gate = event.gate.unwrap_or(0.0);
            let mut refused = 0u32;
            for &(id, chain) in &event.orbit_mods {
                if !orbit.set_mod(id, chain, mod_gate) {
                    refused += 1;
                }
            }
            // A refused chain is motion silently lost, so surface it next to the
            // other drop counters instead of swallowing it.
            #[cfg(feature = "native")]
            if refused > 0 {
                self.metrics
                    .dropped_orbit_mods
                    .fetch_add(refused, std::sync::atomic::Ordering::Relaxed);
            }
            // wasm has no metrics sink; the count is still computed above so the
            // loop body stays identical across configs.
            #[cfg(not(feature = "native"))]
            let _ = refused;
        }

        let v = &mut self.voices[idx];

        // Statics displace any active ModChain on the same param; inline
        // mods below may then install a fresh chain.
        for &id in &event.static_ids {
            v.clear_mod(id);
        }

        // --- Pitch ---
        copy_opt!(event, v.params, detune, speed, glide);
        if let Some(freq) = event.freq {
            if mode != EventMode::New && v.params.glide > 0.0 {
                // Portamento: slew from the current effective pitch.
                v.set_mod(
                    ParamId::Freq,
                    ModChain::Slew {
                        target: freq,
                        freq: 1.0 / v.params.glide,
                        curve: ModCurve::Exponential,
                    },
                );
            } else {
                v.params.freq = freq;
            }
        }
        if let Some(stretch) = event.stretch {
            v.params.stretch = stretch.max(0.0);
        }
        copy_opt!(event, v.params, grain, spray, dens);
        // --- Source ---
        if let Some(source) = parsed_source {
            v.params.sound = source;
            // A re-sounded voice drops any inherited exclusive class; the GM
            // block below re-sets it for soundfont notes.
            #[cfg(feature = "soundfont")]
            {
                v.exclusive_class = 0;
            }
        }

        // Arf patch playback: hand the voice a pooled Vm. A retrigger of the
        // same patch keeps its Vm — graph state persists, and the gate lane
        // re-opens if the voice was releasing (a retrigger of a still-held
        // voice sees no gate edge, so patch-internal envelopes don't re-fire;
        // doux's own VCA re-articulates either way). A different patch
        // returns the old Vm to its pool and takes a fresh one. When the
        // dispatch gate found the pool dry it only admitted a same-patch
        // retrigger, so an empty `take_vm` can leave `patch` unset — the
        // source arm renders that as silence.
        match patch_entry {
            Some(entry) => {
                let same = v
                    .patch
                    .as_ref()
                    .is_some_and(|p| Arc::ptr_eq(&p.entry, &entry));
                if !same {
                    if let Some(old) = v.patch.take() {
                        self.patch_registry.retire(old);
                    }
                    v.patch = entry.take_vm().map(|vm| patch::VoicePatch::new(entry, vm));
                }
                v.params.sound = Source::Arf;
            }
            None => {
                // The event re-sounds this voice with a non-arf source:
                // release its Vm now — waiting for voice death would strand
                // it on a held voice and starve the patch's pool.
                if v.patch.is_some() {
                    #[cfg(feature = "native")]
                    let resounded =
                        parsed_source.is_some() || registry_sample_data.is_some() || has_web_sample;
                    #[cfg(not(feature = "native"))]
                    let resounded =
                        parsed_source.is_some() || loaded_sample.is_some() || has_web_sample;
                    if resounded {
                        if let Some(old) = v.patch.take() {
                            self.patch_registry.retire(old);
                        }
                    }
                }
            }
        }
        // Voice insert (`fx/<name>`): "off" clears; a same-entry re-send
        // keeps the running Vm (insert state persists across retrigger,
        // like the source rule); a different patch swaps. A dry take leaves
        // the slot unset — the stage is simply skipped. A non-arf resound
        // does NOT clear the insert: it is orthogonal to the source.
        if event.fx.as_deref() == Some("off") {
            if let Some(old) = v.fx_patch.take() {
                self.patch_registry.retire(old);
            }
        } else if let Some(entry) = fx_entry {
            let same = v
                .fx_patch
                .as_ref()
                .is_some_and(|p| Arc::ptr_eq(&p.entry, &entry));
            if !same {
                if let Some(old) = v.fx_patch.take() {
                    self.patch_registry.retire(old);
                }
                v.fx_patch = entry.take_vm().map(|vm| patch::VoicePatch::new(entry, vm));
            }
        }
        // Named patch params (`p:name`). A note (new or retriggered) starts
        // from the declared defaults — the script re-states what it wants on
        // every event, so a deleted param-set audibly reverts. A sourceless
        // update writes only what it names: it must never re-assert defaults
        // on a held voice. Names the program doesn't declare are ignored,
        // like any unknown wire key.
        if let Some(program) = v.patch.as_ref().map(|p| Arc::clone(p.entry.program())) {
            if mode != EventMode::Update {
                for (i, &(_, default)) in program.params().iter().enumerate() {
                    let lane = (arf::graph::PARAM_BASE + i) as u8;
                    v.clear_mod(ParamId::PatchLane(lane));
                    if let Some(p) = v.patch.as_mut() {
                        p.control[lane as usize] = default;
                    }
                }
            }
            for (name, value) in &event.patch_params {
                let Some(lane) = program.param_lane(name) else {
                    continue;
                };
                let id = ParamId::PatchLane(lane as u8);
                match value {
                    PatchParamValue::Value(x) => {
                        v.clear_mod(id);
                        if let Some(p) = v.patch.as_mut() {
                            p.control[lane as usize] = *x;
                        }
                    }
                    PatchParamValue::Chain(chain) => v.set_mod(id, *chain),
                }
            }
        }
        copy_opt!(event, v.params, pw, spread);
        if let Some(wave) = event.wave {
            v.params.wave = wave.clamp(0.0, 1.0);
        }
        if let Some(sub) = event.sub {
            v.params.sub = sub.clamp(0.0, 1.0);
        }
        if let Some(sub_oct) = event.sub_oct {
            v.params.sub_oct = sub_oct.clamp(1, 3);
        }
        if let Some(sub_wave) = event.sub_wave {
            v.params.sub_wave = sub_wave;
        }
        if let Some(sync_ratio) = event.sync_ratio {
            v.params.sync_ratio = sync_ratio.clamp(0.0, 64.0);
        }
        if let Some(sync_phase) = event.sync_phase {
            v.params.sync_phase = sync_phase.clamp(0.0, 1.0);
        }
        if let Some(sync_mode) = event.sync_mode {
            v.params.sync_mode = sync_mode;
        }
        if let Some(size) = event.size {
            v.params.shape.size = size.min(256);
        }
        if let Some(warp) = event.warp {
            v.params.shape.warp = warp.clamp(-1.0, 1.0);
        }
        if let Some(mirror) = event.mirror {
            v.params.shape.mirror = mirror.clamp(0.0, 1.0);
        }
        if let Some(harmonics) = event.harmonics {
            v.params.harmonics = harmonics.clamp(0.01, 0.999);
        }
        if let Some(timbre) = event.timbre {
            v.params.timbre = timbre.clamp(0.01, 0.999);
        }
        if let Some(morph) = event.morph {
            v.params.morph = morph.clamp(0.01, 0.999);
        }
        copy_opt_some!(event, v.params, cut);

        // Wavetable scan parameter
        if let Some(scan) = event.scan {
            v.params.scan = scan.clamp(0.0, 1.0);
        }
        if let Some(wtlen) = event.wtlen {
            v.params.wt_cycle_len = wtlen;
        }

        // GM soundfont sample setup
        #[cfg(feature = "soundfont")]
        let is_gm = gm_resolved.is_some();
        #[cfg(feature = "soundfont")]
        if let Some(gm) = gm_resolved {
            let mut rs = RegistrySample::new(None, gm.data, 0.0, 1.0);
            rs.root_freq = gm.root_freq;
            rs.scale_tuning = gm.scale_tuning;
            rs.sr_ratio = gm.sr_ratio;
            rs.attenuation = gm.attenuation;
            rs.loop_until_release = gm.loop_until_release;
            if gm.looping {
                rs.set_loop(gm.loop_start, gm.loop_end);
            }
            v.registry_sample = Some(rs);
            v.exclusive_class = gm.exclusive_class;
            if event.freq.is_none() {
                v.params.freq = 261.626;
            }
            if event.envdelay.is_none() {
                v.params.envdelay = gm.delay;
            }
            if event.attack.is_none() {
                v.params.attack = gm.attack;
            }
            if event.hold.is_none() {
                v.params.hold = gm.hold;
            }
            if event.decay.is_none() {
                v.params.decay = gm.decay;
            }
            if event.sustain.is_none() {
                v.params.sustain = gm.sustain;
            }
            if event.release.is_none() {
                v.params.release = gm.release;
            }
            if event.pan.is_none() {
                v.params.pan = gm.pan;
            }
            if event.lpf.is_none() {
                if let Some(fc) = gm.filter_fc {
                    v.params.lpf = Some(fc);
                    v.params.lpq = gm.filter_q;
                }
            }
            // Vibrato LFO from the zone (SF2 vibLfoToPitch), only when present.
            if gm.vib_depth > 0.0 {
                v.params.vib = gm.vib_rate;
                v.params.vibmod = gm.vib_depth;
            }
        }

        // Sample playback via lock-free registry (native)
        #[cfg(feature = "native")]
        if let Some((sample_name, sample_data)) = registry_sample_data {
            // Use Wavetable mode if scan param present (static or modulated), otherwise Sample
            let has_scan =
                event.scan.is_some() || event.mods.iter().any(|(id, _)| *id == ParamId::Scan);
            v.params.sound = if has_scan {
                Source::Wavetable
            } else {
                Source::Sample
            };
            let (begin, end) = event.resolve_range();
            let frame_count = sample_data.total_frames;
            v.registry_sample = Some(RegistrySample::new(
                Some(sample_name),
                sample_data,
                begin,
                end,
            ));
            if let Some((name_b, data_b)) = registry_sample_data_b {
                v.registry_sample_b = Some(RegistrySample::new(Some(name_b), data_b, begin, end));
                v.sample_blend = sample_blend;
            } else {
                v.registry_sample_b = None;
                v.sample_blend = 0.0;
            }
            if event.freq.is_none() {
                v.params.freq = 261.626;
            }
            if let Some(target_dur) = event.fit {
                let sample_dur = frame_count as f32 * (end - begin) / self.sr;
                let sp = sample_dur / target_dur;
                if sp.is_finite() {
                    v.params.speed = sp;
                }
            }
        } else if event.begin.is_some() || event.end.is_some() || event.slice.is_some() {
            #[cfg(feature = "native")]
            {
                if let Some(ref mut rs) = v.registry_sample {
                    let (begin, end) = event.resolve_range();
                    rs.update_range(Some(begin), Some(end));
                }
                if let Some(ref mut rs) = v.registry_sample_b {
                    let (begin, end) = event.resolve_range();
                    rs.update_range(Some(begin), Some(end));
                }
            }
        }

        // Sample playback via legacy pool (WASM only)
        #[cfg(not(feature = "native"))]
        if let Some(sample_idx) = loaded_sample {
            if let Some(info) = self.samples.get(sample_idx) {
                use sampling::FileSource;
                // Use Wavetable mode if scan param present (static or modulated), otherwise Sample
                let has_scan =
                    event.scan.is_some() || event.mods.iter().any(|(id, _)| *id == ParamId::Scan);
                v.params.sound = if has_scan {
                    Source::Wavetable
                } else {
                    Source::Sample
                };
                let (begin, end) = event.resolve_range();
                v.file_source = Some(FileSource::new(sample_idx, info.frames, begin, end));
                if event.freq.is_none() {
                    v.params.freq = 261.626;
                }
                if let Some(target_dur) = event.fit {
                    let sample_dur = info.frames as f32 * (end - begin) / self.sr;
                    let sp = sample_dur / target_dur;
                    if sp.is_finite() {
                        v.params.speed = sp;
                    }
                }
            }
        } else if event.begin.is_some() || event.end.is_some() || event.slice.is_some() {
            #[cfg(not(feature = "native"))]
            if let Some(ref mut fs) = v.file_source {
                if let Some(info) = self.samples.get(fs.sample_idx) {
                    let (begin, end) = event.resolve_range();
                    fs.update_range(info.frames, Some(begin), Some(end));
                }
            }
        }

        // Web sample playback (set by JavaScript)
        if let (Some(offset), Some(frames)) = (event.file_pcm, event.file_frames) {
            use sampling::WebSampleSource;
            let (begin, end) = event.resolve_range();
            // Use Wavetable mode if scan param present (static or modulated), otherwise WebSample
            let has_scan =
                event.scan.is_some() || event.mods.iter().any(|(id, _)| *id == ParamId::Scan);
            v.params.sound = if has_scan {
                Source::Wavetable
            } else {
                Source::WebSample
            };
            v.web_sample = Some(WebSampleSource::new(
                offset,
                frames as u32,
                event.file_channels.unwrap_or(1),
                event.file_freq.unwrap_or(65.406),
                begin,
                end,
            ));
            if event.freq.is_none() {
                v.params.freq = 261.626;
            }
        }

        // --- Gain ---
        copy_opt!(event, v.params, gain, postgain, velocity, pan, gate);
        // GM applies velocity as the SF2 concave amplitude curve (folded into the
        // sample attenuation in `resolve_gm`); neutralize the linear VCA velocity
        // so it isn't applied twice.
        #[cfg(feature = "soundfont")]
        if is_gm {
            v.params.velocity = 1.0;
        }

        // --- Gain Envelope ---
        if mode == EventMode::Update {
            // Live update: retarget only the stages the event names — no
            // drum defaults, no `init_envelope` backfill stomping the rest.
            copy_opt!(event, v.params, envdelay, attack, hold, decay, sustain, release);
            // Gate is live: re-arm the running envelope so an explicit gate on a
            // sourceless update can end a held voice (or extend / re-hold it).
            if let Some(g) = event.gate {
                v.dahdsr.set_gate(g);
            }
        } else {
            let (att, dec, sus, rel) = if let Some((d_freq, d_att, d_dec, d_sus, d_rel)) =
                v.params.sound.drum_defaults()
            {
                if event.freq.is_none() {
                    v.params.freq = d_freq;
                }
                // Exponential-feel VCA: a linear-in-dB tail so `dec` reads as
                // "time to silence". Spawn-only; the Update branch leaves a
                // sounding drum's curve alone, `reset` restores the 2.0 default.
                v.dahdsr.set_decay_curve(5.0);
                (
                    event.attack.or(Some(d_att)),
                    event.decay.or(Some(d_dec)),
                    event.sustain.or(Some(d_sus)),
                    event.release.or(Some(d_rel)),
                )
            } else {
                (event.attack, event.decay, event.sustain, event.release)
            };
            let gain_env = init_envelope(None, event.envdelay, att, event.hold, dec, sus, rel);
            if gain_env.active {
                v.params.envdelay = gain_env.dly;
                v.params.attack = gain_env.att;
                v.params.hold = gain_env.hld;
                v.params.decay = gain_env.dec;
                v.params.sustain = gain_env.sus;
                v.params.release = gain_env.rel;
            }
        }

        // --- Filters ---
        copy_opt_some!(event, v.params, lpf);
        copy_opt!(event, v.params, lpq);
        copy_opt_some!(event, v.params, hpf);
        copy_opt!(event, v.params, hpq);
        copy_opt_some!(event, v.params, bpf);
        copy_opt!(event, v.params, bpq);
        copy_opt_some!(event, v.params, slpf);
        copy_opt!(event, v.params, slpq);
        copy_opt_some!(event, v.params, shpf);
        copy_opt!(event, v.params, shpq);
        copy_opt_some!(event, v.params, sbpf);
        copy_opt!(event, v.params, sbpq);
        copy_opt_some!(event, v.params, llpf);
        copy_opt!(event, v.params, llpq);
        copy_opt_some!(event, v.params, lhpf);
        copy_opt!(event, v.params, lhpq);
        copy_opt_some!(event, v.params, lbpf);
        copy_opt!(event, v.params, lbpq);

        // --- Modulation ---
        copy_opt!(event, v.params, vib, vibmod, vibshape);
        copy_opt!(event, v.params, fm, fmh, fmshape, fm2, fm2h, fmpivot, fmfb, fmloop);
        copy_opt!(event, v.params, am, amdepth, amshape);
        copy_opt!(event, v.params, rm, rmdepth, rmshape);

        // --- Effects ---
        copy_opt!(
            event,
            v.params,
            phaser,
            phaserdepth,
            phasersweep,
            phasercenter
        );
        copy_opt!(
            event,
            v.params,
            flanger,
            flangerdepth,
            flangerfeedback,
            flangermode
        );
        copy_opt!(event, v.params, fshift);
        copy_opt!(event, v.params, pshift, pshiftwin);
        copy_opt!(event, v.params, wah, wahpeak, wahsens, wahmanual);
        copy_opt!(event, v.params, vinyl, vinylwow, vinylnoise, vinyltone, vinyltype);
        copy_opt!(event, v.params, smear, smearfreq, smearfb);
        copy_opt!(
            event,
            v.params,
            chorus,
            chorusdepth,
            chorusdelay,
            chorustype
        );
        copy_opt_some!(event, v.params, coarse, crush, fold, wrap, distort);
        copy_opt!(
            event,
            v.params,
            distortvol,
            distortmode,
            distortasym,
            foldmode
        );
        copy_opt!(event, v.params, width, haas);
        copy_opt_some!(event, v.params, superpan);
        copy_opt!(event, v.params, superwidth);
        if let Some(set) = event.speakers {
            v.params.speakers = set;
        }
        copy_opt!(event, v.params, eqlo, eqmid, eqhi, eqlofreq, eqmidfreq, eqmidq, eqhifreq, tilt);

        // --- Routing (orbit FX state lives on the orbit, not the voice) ---
        copy_opt!(event, v.params, orbit);

        // Live input channel
        copy_opt_some!(event, v.params, inchan);

        // Install inline parameter modulations
        for (id, chain) in &event.mods {
            v.set_mod(*id, *chain);
        }

        v.sync_source_state();
    }

    /// Frees voice at slot `i` by swapping the last active voice into `i` and
    /// decrementing `active_voices`. Associated fn so the engine can free
    /// voices from inside `gen_block` while other `&mut self.*` fields
    /// (orbits, scratch) are concurrently borrowed via split-borrow.
    #[inline]
    fn free_voice_in(voices: &mut [Voice], active_voices: &mut usize, i: usize) {
        if *active_voices > 0 {
            *active_voices -= 1;
            voices.swap(i, *active_voices);
        }
    }

    fn process_schedule(&mut self) {
        let tolerance = (0.02 * self.sr as f64) as u64;
        loop {
            let t = match self.schedule.peek_tick() {
                Some(t) if t <= self.tick => t,
                _ => return,
            };

            let diff = self.tick - t;
            let event = match self.schedule.pop_front() {
                Some(e) => e,
                None => return,
            };

            if diff < tolerance {
                self.process_event(&event);
            } else {
                #[cfg(feature = "native")]
                self.metrics
                    .dropped_events
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            self.retire_event(event);
        }
    }

    /// Swap a NaN-latched patch Vm for a fresh pooled one, retiring the poisoned
    /// one through the off-RT reaper. `poisoned` is read (Copy) before the slot's
    /// mutable borrow so flag and slot stay disjoint. Fresh-first: a dry pool
    /// leaves the slot untouched rather than stranding it silent. A successful
    /// swap advances the engine-wide cooldown by one second. Sticky user params
    /// live on the `VoicePatch`, not the Vm, so they survive the swap.
    #[inline]
    fn heal_patch(
        poisoned: bool,
        slot: &mut Option<crate::patch::VoicePatch>,
        registry: &crate::patch::PatchRegistry,
        tick: u64,
        heal_sr: u64,
        next_heal_tick: &mut u64,
    ) {
        if !poisoned || tick < *next_heal_tick {
            return;
        }
        let Some(p) = slot.as_mut() else { return };
        let Some(fresh) = p.entry.take_vm() else {
            return;
        };
        let old = std::mem::replace(&mut p.vm, fresh);
        registry.retire_vm(std::sync::Arc::clone(&p.entry), old);
        *next_heal_tick = tick + heal_sr;
    }

    /// Phase F: per-chunk voice + orbit + final-mix pass.
    ///
    /// `start` is the absolute sample offset within the CPAL buffer; `n` is the
    /// chunk length (≤ `dsp_block_size` ≤ `MAX_BLOCK`); `total` is the full
    /// buffer length used for per-orbit recorder addressing.
    ///
    /// Layout:
    /// 1. Clear `orbit.bus[..n]` for every orbit.
    /// 2. One pass over the active voices. Each voice writes `scratch[..written]`
    ///    via `Voice::process_block(n, ...)` (or the split prepare/source/fx
    ///    path under `--features profiling`, which keeps `VoiceSource` and
    ///    `VoiceFx` ns counters separate). The engine accumulates
    ///    `scratch[..written]` into `orbit.bus[..written]`. If `written < n`
    ///    the voice died mid-block and is freed via the swap-and-skip pattern.
    /// 3. `Orbit::process_block(n)` once per orbit — the FX chain runs at block
    ///    rate across the chunk.
    /// 4. Per-frame final mix: per-orbit compressor sidechain (sample-rate by
    ///    design, see `to_do.md:279`), accumulate into the output buffer at
    ///    `(start+f)*output_channels`, then the master chain: DC blocker →
    ///    linked peak limiter → tanh safety clip.
    ///
    /// `#[inline(never)]` — large function; inlining at the (single) call site
    /// in `process_block` would not shrink the call.
    #[allow(unused_variables, clippy::too_many_arguments)]
    #[inline(never)]
    fn gen_block(
        &mut self,
        output: &mut [f32],
        start: usize,
        total: usize,
        n: usize,
        web_pcm: &[f32],
        live_input: &[f32],
        voice_source_ns: &mut u64,
        voice_fx_ns: &mut u64,
        orbit_fx_ns: &mut u64,
        final_mix_ns: &mut u64,
    ) {
        if n == 0 {
            return;
        }

        // Split-borrow: `gen_block` touches voices, orbits, scratch and (on
        // native) the recorder bus concurrently. Destructure at the top so the
        // borrow checker treats each field independently.
        let isr = self.isr;
        let bps = self.tempo_bps;
        let input_channels = self.input_channels;
        let output_channels = self.output_channels;
        let room_gain = self.room_gain;
        let comp_gains = &mut self.comp_gains;
        let master_dc = &mut self.master_dc;
        let master_dc_coeff = self.master_dc_coeff;
        let limiter = &mut self.limiter;
        let master_gain = self.master_gain;
        let prev_master_gain = &mut self.prev_master_gain;
        let bass_mono_coeff = self.bass_mono_coeff;
        let bass_mono_on = self.bass_mono_hz > 0.0;
        let bass_mono_lp = &mut self.bass_mono_lp;
        #[cfg(feature = "native")]
        let rec_orbit = self.recorder.target_orbit();

        // Step 1: clear orbit buses and the superpan accumulator for this chunk.
        for orbit in self.orbits.iter_mut() {
            orbit.clear_bus();
        }
        // Lazy: only a chunk that actually routed a superpan voice dirties the
        // accumulator. Clear the FULL buffer so the clean flag keeps meaning
        // "all zero" even when a later chunk is larger than this one.
        let superpan_acc = &mut self.superpan_acc;
        let superpan_used = &mut self.superpan_acc_used;
        if *superpan_used {
            superpan_acc.fill(0.0);
            *superpan_used = false;
        }

        // Step 2: voice loop. One pass per chunk.
        let voices = &mut self.voices;
        let orbits = &mut self.orbits;
        let active_voices = &mut self.active_voices;
        let patch_registry = &self.patch_registry;
        // NaN-heal cooldown state (§ Step 4). `heal_now` is this chunk's tick;
        // `next_heal_tick` gates heals to ~one per second engine-wide.
        let heal_now = self.tick;
        let heal_sr = self.sr as u64;
        let next_heal_tick = &mut self.next_heal_tick;
        #[cfg(not(feature = "native"))]
        let pool = self.sample_pool.data.as_slice();
        #[cfg(not(feature = "native"))]
        let samples_slice = self.samples.as_slice();

        // Superpan gains scratch, reused across voices: `panaz_gains` fully writes
        // `[..num]` and only `[..num]` is read, so one init per chunk suffices.
        let mut gains = [0.0f32; superpan::MAX_SUPERPAN_NODES];
        let mut i = 0;
        while i < *active_voices {
            let voice = &mut voices[i];

            // Latch the transport tempo into the patch planes for this chunk
            // (`Op::Control` reads are block-invariant, like the param lanes).
            if let Some(p) = voice.patch.as_mut() {
                p.control[arf::graph::BPS_LANE] = bps;
            }
            if let Some(p) = voice.fx_patch.as_mut() {
                p.control[arf::graph::BPS_LANE] = bps;
            }

            #[cfg(all(feature = "native", feature = "profiling"))]
            let written = {
                use std::time::Instant;
                let dsp_start = Instant::now();
                let w = voice.process_block(n, isr, web_pcm, start, live_input, input_channels);
                *voice_source_ns += dsp_start.elapsed().as_nanos() as u64;
                let _ = voice_fx_ns;
                w
            };

            #[cfg(all(feature = "native", not(feature = "profiling")))]
            let written = voice.process_block(n, isr, web_pcm, start, live_input, input_channels);

            #[cfg(not(feature = "native"))]
            let written = voice.process_block(
                n,
                isr,
                pool,
                samples_slice,
                web_pcm,
                start,
                live_input,
                input_channels,
            );

            // Route this voice: `superpan` rings its (stereo) dry around the
            // chosen output PAIRS into the N-wide accumulator (bypassing orbit
            // FX); otherwise it feeds its orbit bus (unchanged stereo path).
            //
            // Ring nodes are stereo pairs: node p -> channels (2p, 2p+1). The
            // voice's L/R is preserved into each pair, scaled by the PanAz gain.
            // `speakers` holds 1-based pair indices (empty = all pairs).
            if let Some(pos) = voice.params.superpan {
                let set = &voice.params.speakers;
                let num_pairs = output_channels / 2;
                let num = if set.is_empty() { num_pairs } else { set.len() }
                    .min(superpan::MAX_SUPERPAN_NODES);
                superpan::panaz_gains(num, pos, voice.params.superwidth, &mut gains);
                *superpan_used = true;
                for f in 0..written {
                    let l = voice.scratch[f][0];
                    let r = voice.scratch[f][1];
                    let base = f * output_channels;
                    for (k, &gain) in gains.iter().enumerate().take(num) {
                        let pair = if set.is_empty() { k } else { set.get(k) };
                        let c0 = pair * 2;
                        if c0 + 1 < output_channels {
                            superpan_acc[base + c0] += l * gain;
                            superpan_acc[base + c0 + 1] += r * gain;
                        }
                    }
                }
                // Also feed the voice's stereo dry into its orbit's FX send so the
                // wet returns to the room — only when that orbit has FX enabled.
                let orbit = &mut orbits[voice.params.orbit % MAX_ORBITS];
                if orbit.has_any_fx() {
                    orbit.has_fx_send = true;
                    orbit.fx_send_used = true;
                    for f in 0..written {
                        orbit.fx_send[f][0] += voice.scratch[f][0];
                        orbit.fx_send[f][1] += voice.scratch[f][1];
                    }
                }
            } else {
                // Accumulate this voice's output into its orbit bus.
                let orbit_idx = voice.params.orbit % MAX_ORBITS;
                let orbit = &mut orbits[orbit_idx];
                for f in 0..written {
                    for c in 0..CHANNELS {
                        orbit.bus[f][c] += voice.scratch[f][c];
                    }
                }
                orbit.has_pan_dry = true;
                orbit.bus_used = true;
            }

            if written < n {
                // Voice died mid-block; return any arf Vms to their pools,
                // then swap last active into slot `i` and re-check the new
                // occupant. The takes keep the invariant that slots beyond
                // `active_voices` never hold a patch.
                if let Some(vp) = voices[i].patch.take() {
                    patch_registry.retire(vp);
                }
                if let Some(vp) = voices[i].fx_patch.take() {
                    patch_registry.retire(vp);
                }
                Self::free_voice_in(voices, active_voices, i);
                continue;
            }

            // Voice survived the block: heal a NaN-latched source or insert Vm.
            let voice = &mut voices[i];
            Self::heal_patch(
                voice.patch_poisoned,
                &mut voice.patch,
                patch_registry,
                heal_now,
                heal_sr,
                next_heal_tick,
            );
            Self::heal_patch(
                voice.fx_patch_poisoned,
                &mut voice.fx_patch,
                patch_registry,
                heal_now,
                heal_sr,
                next_heal_tick,
            );

            i += 1;
        }

        // Step 3: orbit FX chain — block-rate.
        #[cfg(all(feature = "native", feature = "profiling"))]
        let orbit_fx_start = std::time::Instant::now();
        for orbit in orbits.iter_mut() {
            if let Some(p) = orbit.patch.as_mut() {
                p.control[arf::graph::BPS_LANE] = bps;
            }
            orbit.process_block(n);
            Self::heal_patch(
                orbit.patch_poisoned,
                &mut orbit.patch,
                patch_registry,
                heal_now,
                heal_sr,
                next_heal_tick,
            );
        }
        #[cfg(all(feature = "native", feature = "profiling"))]
        {
            *orbit_fx_ns += orbit_fx_start.elapsed().as_nanos() as u64;
        }

        // Step 4: per-frame final mix. Compressor envelope follower is
        // sample-rate by design (`to_do.md:279`) so this loop stays per-sample.
        #[cfg(all(feature = "native", feature = "profiling"))]
        let final_mix_start = std::time::Instant::now();

        let num_pairs = output_channels / 2;

        // Orbit-major passes. Each output slot still receives its contributions
        // in the same order as the old frame-major loop — orbits ascending, then
        // superpan, then room wet, then soft-clip — so the f32 sums are
        // bit-identical while everything block-constant (compressor coeffs,
        // routing flags) is hoisted out of the per-frame work.

        // Pass 0: clear this chunk's destination slots (contiguous region).
        output[start * output_channels..(start + n) * output_channels].fill(0.0);

        #[cfg(feature = "native")]
        let orbit_rec_bus = &mut self.orbit_rec_bus;

        // Pass 1: non-room orbits onto their stereo pair. A room-routed orbit
        // contributes only its wet, spread in Pass 3, so it skips the pair
        // mapping here — but its compressor and recorder run there, not never.
        for oi in 0..MAX_ORBITS {
            if orbits[oi].room_active {
                continue;
            }
            let pair_offset = (oi % num_pairs) * 2;
            let comp_amount = orbits[oi].comp.params.amount;

            // Idle orbit: provably all-zero bus contributes nothing. Cannot
            // skip when the compressor is engaged (its envelope must keep
            // following the sidechain through silence) or when the recorder
            // captures this orbit (its rows must be written every frame).
            #[cfg(feature = "native")]
            let rec_this = rec_orbit == Some(oi);
            #[cfg(not(feature = "native"))]
            let rec_this = false;
            if !orbits[oi].bus_used && comp_amount == 0.0 && !rec_this {
                continue;
            }

            if orbit_comp_gains(orbits, oi, isr, n, comp_gains) {
                let orbit = &orbits[oi];
                for f in 0..n {
                    let gain = comp_gains[f];
                    let orbit_frame = orbit.bus[f];
                    let base_idx = (start + f) * output_channels;
                    output[base_idx + pair_offset] += orbit_frame[0] * gain;
                    output[base_idx + pair_offset + 1] += orbit_frame[1] * gain;
                    #[cfg(feature = "native")]
                    if rec_orbit == Some(oi) {
                        let bus_idx = (oi * total + start + f) * CHANNELS;
                        orbit_rec_bus[bus_idx] = orbit_frame[0] * gain;
                        orbit_rec_bus[bus_idx + 1] = orbit_frame[1] * gain;
                    }
                }
            } else {
                let orbit = &orbits[oi];
                for f in 0..n {
                    let orbit_frame = orbit.bus[f];
                    let base_idx = (start + f) * output_channels;
                    output[base_idx + pair_offset] += orbit_frame[0];
                    output[base_idx + pair_offset + 1] += orbit_frame[1];
                }
                #[cfg(feature = "native")]
                if rec_orbit == Some(oi) {
                    for f in 0..n {
                        let orbit_frame = orbit.bus[f];
                        let bus_idx = (oi * total + start + f) * CHANNELS;
                        orbit_rec_bus[bus_idx] = orbit_frame[0];
                        orbit_rec_bus[bus_idx + 1] = orbit_frame[1];
                    }
                }
            }
        }

        // Pass 2: superpan dry (already N-wide & panned). Skipped entirely when
        // no voice routed through the accumulator this chunk (it is all-zero).
        if *superpan_used {
            for f in 0..n {
                let base_idx = (start + f) * output_channels;
                let acc_base = f * output_channels;
                for c in 0..output_channels {
                    output[base_idx + c] += superpan_acc[acc_base + c];
                }
            }
        }

        // Pass 3: spread each room-routed orbit's FX wet diffusely across the
        // room. The compressor and the recorder run here for these orbits: they
        // are skipped by Pass 1's pair mapping, not exempt from the rest.
        for oi in 0..MAX_ORBITS {
            if !orbits[oi].room_active {
                continue;
            }
            let comp_on = orbit_comp_gains(orbits, oi, isr, n, comp_gains);
            let orbit = &orbits[oi];
            for f in 0..n {
                let gain = if comp_on { comp_gains[f] } else { 1.0 };
                let w0 = orbit.fx_wet[f][0] * gain;
                let w1 = orbit.fx_wet[f][1] * gain;
                let base_idx = (start + f) * output_channels;
                for c in 0..output_channels {
                    let s = if c % 2 == 0 { w0 } else { w1 };
                    output[base_idx + c] += s * room_gain;
                }
                // The room orbit's stereo analogue for the recorder is its wet
                // pair, pre-spread — what Pass 1 would have captured.
                #[cfg(feature = "native")]
                if rec_orbit == Some(oi) {
                    let bus_idx = (oi * total + start + f) * CHANNELS;
                    orbit_rec_bus[bus_idx] = w0;
                    orbit_rec_bus[bus_idx + 1] = w1;
                }
            }
        }

        // Pass 4: master chain — per-channel DC blocker (~10 Hz one-pole HP),
        // master level, linked peak limiter (instant attack, ~100 ms release),
        // then tanh safety clip. The limiter computes ONE gain per frame from
        // the peak across all channels so the multichannel image never shifts.
        // All state lives on the Engine: gen_block runs once per inner chunk
        // and must not reset it.
        // Bass mono, ahead of everything else in the master chain so the DC
        // blocker and the limiter see the finished image. Below the corner the
        // stereo difference is discarded and both channels carry the centre, so
        // a summed sub array cannot phase-cancel the low end. First-order split
        // per channel; `out_l + out_r == l + r` exactly, so the mono sum of the
        // mix is untouched and only the low-band difference goes away. Skipped
        // whole when off (the default), which is why it is not folded into the
        // loop below.
        if bass_mono_on {
            for f in 0..n {
                let base_idx = (start + f) * output_channels;
                for p in 0..output_channels / 2 {
                    let li = base_idx + p * 2;
                    // Scrub here too: a NaN reaching `bass_mono_lp` would latch
                    // it forever, the same failure the DC blocker guards against.
                    for s in output[li..li + 2].iter_mut() {
                        if !s.is_finite() {
                            *s = 0.0;
                        }
                    }
                    let (l, r) = (output[li], output[li + 1]);
                    bass_mono_lp[p * 2] += bass_mono_coeff * (l - bass_mono_lp[p * 2]);
                    bass_mono_lp[p * 2 + 1] += bass_mono_coeff * (r - bass_mono_lp[p * 2 + 1]);
                    let (low_l, low_r) = (bass_mono_lp[p * 2], bass_mono_lp[p * 2 + 1]);
                    let mono = (low_l + low_r) * 0.5;
                    output[li] = (l - low_l) + mono;
                    output[li + 1] = (r - low_r) + mono;
                }
            }
        }

        let limiter_release = (isr / effects::LIMITER_RELEASE_SECS).min(1.0);
        // Master level ramped across the chunk, so a live move does not step.
        // Constant level => step 0 => exact `frame * gain` (unchanged steady state).
        let gain_prev = *prev_master_gain;
        let gain_step = (master_gain - gain_prev) / n as f32;
        for f in 0..n {
            let base_idx = (start + f) * output_channels;
            let frame = &mut output[base_idx..base_idx + output_channels];
            let level = gain_prev + gain_step * (f as f32 + 1.0);
            let mut peak = 0.0_f32;
            for (c, s) in frame.iter_mut().enumerate() {
                // non-finite in => zero: else master_dc latches NaN forever (silent till restart).
                if !s.is_finite() {
                    *s = 0.0;
                }
                // One-pole HP: track the low band in state, subtract. DC is
                // removed before peak detection so offset doesn't eat limiter
                // headroom or bias the tanh asymmetrically.
                let st = &mut master_dc[c];
                *st += master_dc_coeff * (*s - *st);
                // Master level lands here, BEFORE peak detection: the limiter
                // has to see the boosted signal or it cannot protect against it.
                // (The DC blocker stays upstream so its state is level-independent.)
                let y = (*s - *st) * level;
                *s = y;
                peak = peak.max(y.abs());
            }
            let gain = limiter.process(peak, limiter_release);
            for s in frame.iter_mut() {
                *s = soft_clip_sample(*s * gain);
            }
        }
        *prev_master_gain = master_gain;
        // Denormal hygiene for non-FTZ targets (wasm): flush decayed DC state
        // once per chunk.
        for st in master_dc.iter_mut().take(output_channels) {
            *st = ftz(*st, 1.0e-12);
        }
        if bass_mono_on {
            for st in bass_mono_lp.iter_mut().take(output_channels) {
                *st = ftz(*st, 1.0e-12);
            }
        }

        #[cfg(all(feature = "native", feature = "profiling"))]
        {
            *final_mix_ns += final_mix_start.elapsed().as_nanos() as u64;
        }
    }

    pub fn process_block(&mut self, output: &mut [f32], web_pcm: &[f32], live_input: &[f32]) {
        debug_assert!(
            output.len() <= MAX_BUFFER_FRAMES * self.output_channels,
            "process_block: output ({} samples) exceeds per-callback ceiling ({})",
            output.len(),
            MAX_BUFFER_FRAMES * self.output_channels,
        );

        // Wall-clock for the load gate + `BlockTotal` metric. Permitted on the
        // audio thread per `to_do.md` real-time invariants: resolves via VDSO
        // (`mach_absolute_time` / `clock_gettime(CLOCK_MONOTONIC)`), no kernel
        // transition. Load-bearing for overload-driven voice shedding below.
        #[cfg(feature = "native")]
        let start = std::time::Instant::now();

        // Clamp to the allocation ceiling so a device period larger than
        // `MAX_BUFFER_FRAMES` can never drive the recorder bus out of bounds on
        // the audio thread (the native cpal callback already caps its output
        // slice to the same ceiling, so this never truncates a real block).
        let samples = (output.len() / self.output_channels).min(MAX_BUFFER_FRAMES);

        #[cfg(feature = "native")]
        {
            // orbit_rec_bus is sized for MAX_BUFFER_FRAMES and `samples` is
            // clamped to it above, so this is always satisfied — kept as a dev
            // guard against a future change to either side.
            debug_assert!(
                self.orbit_rec_bus.len() >= MAX_ORBITS * samples * CHANNELS,
                "orbit_rec_bus too small: {} < {}",
                self.orbit_rec_bus.len(),
                MAX_ORBITS * samples * CHANNELS,
            );
        }

        // Pre-block: upgrade registry samples (item 3)
        #[cfg(feature = "native")]
        {
            #[cfg(feature = "profiling")]
            let sample_upgrade_start = std::time::Instant::now();
            for i in 0..self.active_voices {
                if let Some(ref mut rs) = self.voices[i].registry_sample {
                    if let Some(sample_name) = rs.sample_name.as_deref() {
                        if rs.is_head() {
                            if let Some(full) = self.sample_registry.get(sample_name) {
                                if full.frame_count >= full.total_frames {
                                    rs.upgrade(full);
                                }
                            }
                        }
                    }
                }
                if let Some(ref mut rs) = self.voices[i].registry_sample_b {
                    if let Some(sample_name) = rs.sample_name.as_deref() {
                        if rs.is_head() {
                            if let Some(full) = self.sample_registry.get(sample_name) {
                                if full.frame_count >= full.total_frames {
                                    rs.upgrade(full);
                                }
                            }
                        }
                    }
                }
            }
            #[cfg(feature = "profiling")]
            self.metrics.profiler.record_phase(
                ProfilePhase::SampleUpgrade,
                sample_upgrade_start.elapsed().as_nanos() as u64,
            );
        }

        #[cfg(all(feature = "native", feature = "profiling"))]
        let mut schedule_elapsed_ns = 0u64;
        #[cfg(all(feature = "native", feature = "profiling"))]
        let mut voice_source_ns = 0u64;
        #[cfg(all(feature = "native", feature = "profiling"))]
        let mut voice_fx_ns = 0u64;
        #[cfg(all(feature = "native", feature = "profiling"))]
        let mut orbit_fx_ns = 0u64;
        #[cfg(all(feature = "native", feature = "profiling"))]
        let mut final_mix_ns = 0u64;

        // Phase F: block-native end-to-end. The schedule loop stays per-sample
        // (events fire at sample-rate); voices, orbits, and the final mix run
        // at block rate inside `gen_block`. Chunk size = `dsp_block_size`,
        // already clamped to `[1, MAX_BLOCK]` by `DspBlockSize::new`, so
        // `orbit.bus` and `voice.scratch` (both sized `MAX_BLOCK`) never overflow.
        let bs = self.inner_block_size.get();
        debug_assert!(
            bs <= MAX_BLOCK,
            "inner_block_size={bs} > MAX_BLOCK={MAX_BLOCK}"
        );
        let mut chunk_start = 0;
        while chunk_start < samples {
            let n = (samples - chunk_start).min(bs);

            for _ in 0..n {
                #[cfg(all(feature = "native", feature = "profiling"))]
                let schedule_start = std::time::Instant::now();
                self.process_schedule();
                #[cfg(all(feature = "native", feature = "profiling"))]
                {
                    schedule_elapsed_ns += schedule_start.elapsed().as_nanos() as u64;
                }
                self.tick += 1;
            }
            // Wall-clock readout only (telemetry `time_bits` + `get_time`);
            // never read inside the chunk, so once per chunk is equivalent.
            self.time = self.tick as f64 / self.sr as f64;

            // Local block counters; `gen_block` accumulates into them so the
            // per-phase totals across all chunks land in the engine profiler.
            let mut vs = 0u64;
            let mut vf = 0u64;
            let mut ofx = 0u64;
            let mut fm = 0u64;

            self.gen_block(
                output,
                chunk_start,
                samples,
                n,
                web_pcm,
                live_input,
                &mut vs,
                &mut vf,
                &mut ofx,
                &mut fm,
            );

            #[cfg(all(feature = "native", feature = "profiling"))]
            {
                voice_source_ns += vs;
                voice_fx_ns += vf;
                orbit_fx_ns += ofx;
                final_mix_ns += fm;
            }
            #[cfg(not(all(feature = "native", feature = "profiling")))]
            {
                let _ = vs;
                let _ = vf;
                let _ = ofx;
                let _ = fm;
            }

            chunk_start += n;
        }
        #[cfg(all(feature = "native", feature = "profiling"))]
        {
            let profiler = &self.metrics.profiler;
            profiler.record_phase(ProfilePhase::Schedule, schedule_elapsed_ns);
            profiler.record_phase(ProfilePhase::VoiceSource, voice_source_ns);
            profiler.record_phase(ProfilePhase::VoiceFx, voice_fx_ns);
            profiler.record_phase(ProfilePhase::OrbitFx, orbit_fx_ns);
            profiler.record_phase(ProfilePhase::FinalMix, final_mix_ns);
        }

        #[cfg(feature = "native")]
        {
            #[cfg(feature = "profiling")]
            let recorder_start = std::time::Instant::now();
            let n = samples * CHANNELS;
            if let Some(oi) = self.recorder.target_orbit() {
                let start_idx = oi * samples * CHANNELS;
                self.recorder.capture_block(
                    &self.orbit_rec_bus[start_idx..start_idx + n],
                    samples,
                    CHANNELS,
                );
            } else {
                self.recorder
                    .capture_block(output, samples, self.output_channels);
            }
            #[cfg(feature = "profiling")]
            self.metrics.profiler.record_phase(
                ProfilePhase::RecorderCapture,
                recorder_start.elapsed().as_nanos() as u64,
            );
        }

        #[cfg(feature = "native")]
        {
            use std::sync::atomic::Ordering;
            let elapsed_ns = start.elapsed().as_nanos() as u64;
            self.metrics.profiler.record_block(samples);
            self.metrics
                .profiler
                .record_phase(ProfilePhase::BlockTotal, elapsed_ns);
            self.metrics.load.record_sample(elapsed_ns);
            self.metrics
                .active_voices
                .store(self.active_voices as u32, Ordering::Relaxed);
            self.metrics
                .peak_voices
                .fetch_max(self.active_voices as u32, Ordering::Relaxed);
            self.metrics
                .schedule_depth
                .store(self.schedule.len() as u32, Ordering::Relaxed);
            // Single consumer of the limiter's hold: reading resets it, so the
            // readout is "peak reduction since the last block", not cumulative.
            self.metrics.set_limiter_gr(self.limiter.take_reduction());
            self.metrics
                .time_bits
                .store(self.time.to_bits(), Ordering::Relaxed);

            let instant = self.metrics.load.instant_load();
            let smoothed = self.metrics.load.get_load();
            self.load_gate = smoothed > 0.85;

            if instant > 0.95 && self.active_voices > 1 {
                // Phase 1: hard-cut voices already in release (least audible)
                for i in (0..self.active_voices).rev() {
                    if self.voices[i].dahdsr.is_releasing() {
                        self.voices[i].hard_cut();
                    }
                }
                // Phase 2: force-release quietest voices
                if self.active_voices > 4 {
                    let shed_count = (self.active_voices / 4).max(1);
                    for _ in 0..shed_count {
                        if self.active_voices <= 2 {
                            break;
                        }
                        let mut min_idx = 0;
                        let mut min_val = f32::MAX;
                        for i in 0..self.active_voices {
                            let val = self.voices[i].dahdsr.current_val;
                            if val < min_val {
                                min_val = val;
                                min_idx = i;
                            }
                        }
                        self.voices[min_idx].hard_cut();
                    }
                }
            }
        }
    }

    pub fn dsp(&mut self) {
        let mut output = std::mem::take(&mut self.output);
        self.process_block(&mut output, &[], &[]);
        self.output = output;
    }

    pub fn dsp_with_web_pcm(&mut self, web_pcm: &[f32], live_input: &[f32]) {
        let mut output = std::mem::take(&mut self.output);
        self.process_block(&mut output, web_pcm, live_input);
        self.output = output;
    }

    pub fn get_time(&self) -> f64 {
        self.time
    }

    pub fn get_tick(&self) -> u64 {
        self.tick
    }

    /// Set the transport tempo arf patches read as `bps`, in beats per
    /// second. Non-finite or non-positive values are ignored — the lane must
    /// always carry a usable tempo.
    pub fn set_tempo(&mut self, bps: f32) {
        if bps.is_finite() && bps > 0.0 {
            self.tempo_bps = bps;
        }
    }

    /// Set the master output level, linear, clamped to `0..=MASTER_GAIN_MAX`.
    /// Non-finite values are ignored. The move is ramped over the next chunk,
    /// so this is safe to call live. Applied before the limiter's peak
    /// detector, so a boost cannot escape the master safety chain.
    pub fn set_master_gain(&mut self, gain: f32) {
        if gain.is_finite() {
            self.master_gain = gain.clamp(0.0, MASTER_GAIN_MAX);
        }
    }

    /// The current master output level (post-clamp).
    pub fn master_gain(&self) -> f32 {
        self.master_gain
    }

    /// Set the bass-mono crossover corner in Hz; 0 (the default) disables the
    /// stage. Below the corner the stereo image collapses to centre, so a
    /// summed sub array cannot phase-cancel the low end. Clamped to
    /// `0..=BASS_MONO_MAX_HZ`; non-finite values are ignored.
    pub fn set_bass_mono(&mut self, hz: f32) {
        if !hz.is_finite() {
            return;
        }
        let hz = hz.clamp(0.0, BASS_MONO_MAX_HZ);
        if hz == self.bass_mono_hz {
            return;
        }
        self.bass_mono_hz = hz;
        // Same bilinear one-pole as the master DC blocker.
        let w = std::f32::consts::PI * hz / self.sample_rate();
        self.bass_mono_coeff = (2.0 * w) / (1.0 + 2.0 * w);
        // Turning the stage off leaves stale low-band state behind; clear it so
        // re-enabling starts from silence instead of a frozen DC step.
        if hz == 0.0 {
            self.bass_mono_lp = [0.0; MAX_OUTPUT_CHANNELS];
        }
    }

    /// The current bass-mono corner in Hz (0 = off).
    pub fn bass_mono(&self) -> f32 {
        self.bass_mono_hz
    }
    pub fn hush(&mut self) {
        for i in 0..self.active_voices {
            self.voices[i].force_release();
        }
    }

    pub fn panic(&mut self) {
        // Return arf Vms before the slots go inactive — a silenced slot is
        // reused lazily, and a stranded Vm would starve its patch's pool.
        for i in 0..self.active_voices {
            if let Some(vp) = self.voices[i].patch.take() {
                self.patch_registry.retire(vp);
            }
            if let Some(vp) = self.voices[i].fx_patch.take() {
                self.patch_registry.retire(vp);
            }
        }
        self.active_voices = 0;

        // Clear every orbit's FX tail (reverb tanks, delay lines, comp env) so a
        // ringing or NaN-latched orbit goes truly silent — the emergency hatch a
        // `panic`/`reset` promises. The sticky `patch/` effect stays installed:
        // swap its Vm for a fresh pooled one (config survives, state resets) via
        // the Step-4 retire path. Params/levels stay sticky (config, not sound).
        for orbit in self.orbits.iter_mut() {
            orbit.clear_fx_state();
            let Some(p) = orbit.patch.as_mut() else {
                continue;
            };
            let Some(fresh) = p.entry.take_vm() else {
                continue;
            };
            let old = std::mem::replace(&mut p.vm, fresh);
            self.patch_registry
                .retire_vm(std::sync::Arc::clone(&p.entry), old);
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // Voices dying with the engine still hold pooled Vms; send them home
        // so a patch registry that outlives this engine (the device-loss
        // rebuild reuses it) keeps its pools full. Engines drop on control
        // threads (stream teardown), never inside the audio callback.
        for v in &mut self.voices {
            if let Some(vp) = v.patch.take() {
                self.patch_registry.retire(vp);
            }
            if let Some(vp) = v.fx_patch.take() {
                self.patch_registry.retire(vp);
            }
        }
        // Orbit patches hold pooled Vms too. (`panic()` resets their FX *state*
        // but keeps them installed — sticky FX config like `verb_level`, not
        // sounding voices — so this final retire still runs on engine drop.)
        for o in self.orbits.iter_mut() {
            if let Some(vp) = o.patch.take() {
                self.patch_registry.retire(vp);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_clip_keeps_signal_bounded() {
        assert!(soft_clip_sample(2.0).abs() <= 1.0);
        assert!(soft_clip_sample(-1.5).abs() <= 1.0);
    }

    #[test]
    fn soft_clip_is_near_identity_at_low_levels() {
        // tanh slope at origin is 1; near-identity below ~0.3.
        assert!((soft_clip_sample(0.1) - 0.1).abs() < 1e-2);
        assert!((soft_clip_sample(-0.05) + 0.05).abs() < 1e-2);
    }

    // A held voice (gate 0) is ended by a later sourceless update that carries an
    // explicit positive gate, while a gateless tweak leaves it sounding.
    #[cfg(feature = "native")]
    #[test]
    fn live_gate_ends_held_voice() {
        fn render(engine: &mut Engine, seconds: f32) {
            let blocks = ((engine.sample_rate() * seconds) / engine.host_buffer_size() as f32)
                .ceil() as usize;
            for _ in 0..blocks {
                engine.dsp();
            }
        }

        let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));

        // Held voice: infinite sustain, tag 0.
        engine.evaluate("sound/sine/voice/0/gate/0");
        render(&mut engine, 0.2);
        assert_eq!(engine.active_voices(), 1, "held voice should sustain");

        // Sourceless tweak with no gate must NOT end it.
        engine.evaluate("voice/0/lpf/800");
        render(&mut engine, 0.2);
        assert_eq!(
            engine.active_voices(),
            1,
            "a gateless tweak must not end a held voice"
        );

        // Sourceless update with an explicit positive gate ends it.
        engine.evaluate("voice/0/gate/0.05");
        render(&mut engine, 0.2);
        assert_eq!(
            engine.active_voices(),
            0,
            "an explicit positive gate should end a held voice"
        );
    }

    // Closed-loop FM at max drive (fmfb + fmloop both 1, high indices) must
    // stay finite and audible: the loop is delay-averaged, not clamped.
    #[cfg(feature = "native")]
    #[test]
    fn fm_loop_at_max_drive_stays_finite() {
        let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
        engine.evaluate("sound/sine/note/36/fm/10/fmh/1/fm2/10/fm2h/7/fmfb/1/fmloop/1/gate/2");

        let mut out = vec![0.0_f32; engine.host_buffer_size() * engine.output_channels()];
        let blocks = (48_000.0 / engine.host_buffer_size() as f32).ceil() as usize;
        let mut peak = 0.0_f32;
        for _ in 0..blocks {
            engine.process_block(&mut out, &[], &[]);
            for &s in &out {
                assert!(s.is_finite(), "closed-loop FM produced a non-finite sample");
                peak = peak.max(s.abs());
            }
        }
        assert!(peak > 0.0, "voice should be audible");
    }

    // A voice through the SVF lowpass with negative resonance must stay finite
    // and audible. Pre-fix, an unclamped q made Q = 0.5 + q*30 <= 0 (a divide by
    // ~0 plus an unstable filter), poisoning the master DC-blocker and silencing
    // the engine until restart. The .dsp now clamps q, so the last block must
    // still carry signal (filter stayed stable, not zeroed by the master guard
    // after diverging to NaN).
    #[cfg(feature = "native")]
    #[test]
    fn negative_resonance_stays_finite_and_audible() {
        let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
        engine.evaluate("sound/sine/note/48/lpf/1000/lpq/-0.5/gate/2");

        let mut out = vec![0.0_f32; engine.host_buffer_size() * engine.output_channels()];
        let blocks = (48_000.0 / engine.host_buffer_size() as f32).ceil() as usize;
        let mut last_peak = 0.0_f32;
        for b in 0..blocks {
            engine.process_block(&mut out, &[], &[]);
            let mut peak = 0.0_f32;
            for &s in &out {
                assert!(
                    s.is_finite(),
                    "negative resonance produced a non-finite sample"
                );
                peak = peak.max(s.abs());
            }
            if b == blocks - 1 {
                last_peak = peak;
            }
        }
        assert!(
            last_peak > 0.0,
            "filter should stay stable and audible under negative resonance"
        );
    }

    // A voice through the 3-band EQ with a zero mid-Q must stay finite and
    // audible. Pre-fix, eq.dsp divided by `q * sin(...)`, so q = 0 (typed or
    // modulated) gave 1/0 in the mid-peak coefficients, poisoning the master
    // DC-blocker and silencing the engine until restart. The .dsp now clamps q
    // (and the band freqs), so the last block must still carry signal.
    #[cfg(feature = "native")]
    #[test]
    fn eq_zero_q_stays_finite_and_audible() {
        let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
        engine.evaluate("sound/sine/note/48/eqmid/6/eqmidq/0/gate/2");

        let mut out = vec![0.0_f32; engine.host_buffer_size() * engine.output_channels()];
        let blocks = (48_000.0 / engine.host_buffer_size() as f32).ceil() as usize;
        let mut last_peak = 0.0_f32;
        for b in 0..blocks {
            engine.process_block(&mut out, &[], &[]);
            let mut peak = 0.0_f32;
            for &s in &out {
                assert!(s.is_finite(), "zero mid-Q produced a non-finite sample");
                peak = peak.max(s.abs());
            }
            if b == blocks - 1 {
                last_peak = peak;
            }
        }
        assert!(
            last_peak > 0.0,
            "EQ should stay stable and audible under a zero mid-Q"
        );
    }

    // Every Faust effect with a user-controllable frequency / Q / feedback /
    // window must clamp it internally: an out-of-range value (typed or modulated)
    // used to divide by ~0 or drive an unstable recursion diverges to NaN, which
    // latches the master DC-blocker and silences the engine until restart. Each
    // patch drives one effect past its pre-fix singularity; post-fix every sample
    // stays finite and the note stays audible.
    #[cfg(feature = "native")]
    #[test]
    fn faust_effects_clamp_pathological_params() {
        // (what it exercises, patch that hit the pre-fix singularity)
        let cases = [
            (
                "wah negative resonance",
                "sound/saw/note/48/wah/1/wahpeak/-0.5/gate/2",
            ),
            (
                "smear negative freq (tan(t)+1=0)",
                "sound/sine/note/48/smear/1/smearfreq/-12000/gate/2",
            ),
            (
                "phaser feedback >= 1",
                "sound/sine/note/48/phaser/1/phaserdepth/1.5/gate/2",
            ),
            (
                "flanger large-negative feedback",
                "sound/sine/note/48/flanger/1/flangerfeedback/-5/gate/2",
            ),
            (
                "pshift zero window (divide by window)",
                "sound/sine/note/48/pshift/12/pshiftwin/0/gate/2",
            ),
            (
                "comb damp > 1 (pole outside unit circle)",
                "sound/sine/note/48/comb/0.8/combfreq/200/combdamp/5/gate/2",
            ),
            (
                "feedback damp > 1 (pole outside unit circle)",
                "sound/sine/note/48/feedback/0.8/fbdamp/5/gate/2",
            ),
        ];
        for (name, patch) in cases {
            let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
            engine.evaluate(patch);

            let mut out = vec![0.0_f32; engine.host_buffer_size() * engine.output_channels()];
            let blocks = (48_000.0 / engine.host_buffer_size() as f32).ceil() as usize;
            let mut last_peak = 0.0_f32;
            for b in 0..blocks {
                engine.process_block(&mut out, &[], &[]);
                let mut peak = 0.0_f32;
                for &s in &out {
                    assert!(s.is_finite(), "{name}: produced a non-finite sample");
                    peak = peak.max(s.abs());
                }
                if b == blocks - 1 {
                    last_peak = peak;
                }
            }
            assert!(
                last_peak > 0.0,
                "{name}: effect should stay stable and audible"
            );
        }
    }

    // Named patch params: a note starts from the declared defaults, an event's
    // `p:name` writes reach the lane, a sourceless update writes without
    // resetting, and a chain ticks the lane per sample.
    #[cfg(feature = "native")]
    #[test]
    fn patch_params_route_to_the_lane_and_reset_per_note() {
        use arf::graph::PARAM_BASE;

        fn render(engine: &mut Engine, seconds: f32) {
            let blocks = ((engine.sample_rate() * seconds) / engine.host_buffer_size() as f32)
                .ceil() as usize;
            for _ in 0..blocks {
                engine.dsp();
            }
        }
        fn lane(engine: &Engine, voice: usize, lane: usize) -> f32 {
            engine.voices[voice]
                .patch
                .as_ref()
                .expect("voice holds a patch")
                .control[lane]
        }

        let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));

        // `param cutoff 400  notefreq saw cutoff lpf out`, via the graph API.
        let mut g = arf::graph::Graph::new();
        let cut_lane = g.add_param("cutoff".to_string(), 400.0);
        let cut = g.control(cut_lane);
        let nf = g.control(arf::graph::NOTEFREQ_LANE as u32);
        let saw = g.ugen(arf::ugen::lookup("saw").unwrap(), vec![nf]);
        let filt = g.ugen(arf::ugen::lookup("lpf").unwrap(), vec![saw, cut]);
        g.set_outputs(vec![filt]);
        let json = serde_json::to_string(&g).unwrap();
        engine
            .patch_registry
            .install_graph("pp", &json, 48_000.0)
            .unwrap();

        // A note with a static write reaches the lane; an unknown name is ignored.
        engine.evaluate("sound/pp/voice/0/gate/0/p:cutoff/2000/p:nope/1");
        render(&mut engine, 0.05);
        assert_eq!(engine.active_voices(), 1);
        assert_eq!(lane(&engine, 0, PARAM_BASE), 2000.0);

        // A sourceless update writes the lane without ending or resetting.
        engine.evaluate("voice/0/p:cutoff/900");
        render(&mut engine, 0.05);
        assert_eq!(lane(&engine, 0, PARAM_BASE), 900.0);

        // A retrigger without the param re-asserts the declared default.
        engine.evaluate("sound/pp/voice/0/gate/0");
        render(&mut engine, 0.05);
        assert_eq!(lane(&engine, 0, PARAM_BASE), 400.0);

        // A chain rides the lane per sample: after a render the lane sits
        // inside the chain's range, not at the default.
        engine.evaluate("sound/pp/voice/0/gate/0/p:cutoff/3000~5000:2");
        render(&mut engine, 0.05);
        let v = lane(&engine, 0, PARAM_BASE);
        assert!(
            (3000.0..=5000.0).contains(&v),
            "chain did not tick the lane: {v}"
        );
    }

    // === Master output level ===

    #[cfg(feature = "native")]
    fn render_blocks(engine: &mut Engine, seconds: f32) {
        let blocks =
            ((engine.sample_rate() * seconds) / engine.host_buffer_size() as f32).ceil() as usize;
        for _ in 0..blocks {
            engine.dsp();
        }
    }

    #[test]
    fn master_gain_clamps_to_its_range() {
        let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
        engine.set_master_gain(9.0);
        assert_eq!(engine.master_gain(), MASTER_GAIN_MAX);
        engine.set_master_gain(-1.0);
        assert_eq!(engine.master_gain(), 0.0);
        // Non-finite is ignored rather than latched.
        engine.set_master_gain(0.5);
        engine.set_master_gain(f32::NAN);
        assert_eq!(engine.master_gain(), 0.5);
    }

    #[cfg(feature = "native")]
    #[test]
    fn master_gain_zero_silences_the_output() {
        let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
        engine.evaluate("sound/sine/gain/1");
        engine.set_master_gain(0.0);
        render_blocks(&mut engine, 0.2);
        let mut out = vec![0.0f32; engine.host_buffer_size() * 2];
        engine.process_block(&mut out, &[], &[]);
        assert!(
            out.iter().all(|s| *s == 0.0),
            "master gain 0 must mute; peak was {}",
            out.iter().fold(0.0f32, |a, s| a.max(s.abs()))
        );
    }

    // The ordering test, and the reason the master level sits where it does:
    // it is applied BEFORE the limiter's peak detector, so a boost drives the
    // limiter instead of escaping it. Move the multiply after peak detection
    // and this fails with zero reduction. The source is picked to sit just
    // under the ceiling at unity (~0.49) and just over it at 2x.
    #[cfg(feature = "native")]
    #[test]
    fn boosted_master_still_reaches_the_limiter() {
        fn gr_at(master: f32) -> f32 {
            let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
            engine.evaluate("sound/sine/gain/1/postgain/3");
            engine.set_master_gain(master);
            render_blocks(&mut engine, 0.2);
            engine.metrics().take_limiter_gr()
        }

        assert_eq!(gr_at(1.0), 0.0, "this source must not limit at unity");
        assert!(
            gr_at(MASTER_GAIN_MAX) > 0.0,
            "the limiter must see the boosted master"
        );
    }

    // The engine writes gain reduction far faster than a UI reads it, so the
    // readout accumulates a maximum and clears on read. A plain store would let
    // a limiting event land and vanish between two frames.
    #[cfg(feature = "native")]
    #[test]
    fn limiter_gr_survives_until_it_is_read() {
        let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
        engine.evaluate("sound/sine/gain/1/postgain/3");
        engine.set_master_gain(MASTER_GAIN_MAX);
        render_blocks(&mut engine, 0.2);
        // Quiet again: many blocks pass with no reduction at all.
        engine.hush();
        render_blocks(&mut engine, 0.5);
        assert!(
            engine.metrics().take_limiter_gr() > 0.0,
            "the peak must hold across blocks until a reader takes it"
        );
        assert_eq!(
            engine.metrics().take_limiter_gr(),
            0.0,
            "taking it must clear the accumulator"
        );
    }

    #[test]
    fn master_ceiling_is_the_clipped_limiter_threshold() {
        let ceiling = master_ceiling();
        assert!(
            (0.5..1.0).contains(&ceiling),
            "ceiling {ceiling} is not a plausible output bound"
        );
        assert_eq!(ceiling, soft_clip_sample(effects::LIMITER_THRESHOLD));
    }

    // === Bass mono ===

    #[test]
    fn bass_mono_clamps_and_reports_its_corner() {
        let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
        assert_eq!(engine.bass_mono(), 0.0, "off by default");
        engine.set_bass_mono(9_000.0);
        assert_eq!(engine.bass_mono(), BASS_MONO_MAX_HZ);
        engine.set_bass_mono(-5.0);
        assert_eq!(engine.bass_mono(), 0.0);
        engine.set_bass_mono(120.0);
        engine.set_bass_mono(f32::NAN);
        assert_eq!(engine.bass_mono(), 120.0);
    }

    // The load-bearing property: the crossover discards only the low-band
    // *difference*, so what a mono PA hears is unchanged. A club sums to mono in
    // the sub array, and a bass-mono stage that altered that sum would be
    // changing the mix rather than protecting it. The source is kept quiet on
    // purpose: the master tanh downstream is nonlinear, so it redistributes a
    // hot signal slightly when energy moves between channels, and the invariant
    // belongs to the crossover, not to the safety clip.
    #[cfg(feature = "native")]
    #[test]
    fn bass_mono_preserves_the_mono_sum() {
        fn mono_sum(bass_mono: f32) -> Vec<f32> {
            let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
            engine.set_bass_mono(bass_mono);
            engine.evaluate("sound/sine/freq/40/gain/0.1/pan/0");
            render_blocks(&mut engine, 0.3);
            let mut out = vec![0.0f32; engine.host_buffer_size() * 2];
            engine.process_block(&mut out, &[], &[]);
            out.chunks_exact(2).map(|f| f[0] + f[1]).collect()
        }

        let off = mono_sum(0.0);
        let on = mono_sum(200.0);
        assert!(off.iter().any(|s| s.abs() > 1e-4), "test signal was silent");
        for (i, (a, b)) in off.iter().zip(&on).enumerate() {
            assert!(
                (a - b).abs() < 1e-5,
                "frame {i}: bass mono changed the mono sum, {a} vs {b}"
            );
        }
    }

    // === Compressor ===

    // With no `comporbit` the detector is the orbit's own bus, so a bare `comp`
    // glues. It used to default to orbit 0, which meant an unrelated orbit's
    // material ducked yours by accident.
    #[cfg(feature = "native")]
    #[test]
    fn comp_without_comporbit_compresses_itself() {
        fn peak_on_orbit_1(extra: &str) -> f32 {
            let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
            engine.evaluate(&format!("sound/sine/orbit/1/gain/1/postgain/3{extra}"));
            render_blocks(&mut engine, 0.3);
            let mut out = vec![0.0f32; engine.host_buffer_size() * 2];
            engine.process_block(&mut out, &[], &[]);
            out.iter().fold(0.0f32, |a, s| a.max(s.abs()))
        }

        let open = peak_on_orbit_1("");
        // Orbit 0 is silent, so under the old orbit-0 default this would not
        // have compressed at all.
        let glued = peak_on_orbit_1("/comp/1/compthresh/-30/compratio/8");
        assert!(
            glued < open * 0.8,
            "a bare comp must compress this orbit: open {open}, glued {glued}"
        );
    }

    // === Room routing keeps the compressor ===

    // A room-routed orbit used to be skipped wholesale by the stereo-pair pass,
    // which took its compressor and its recorder with it. Pass 3 now runs the
    // same gain helper, so ducking survives the room latch. Measured on the far
    // channels, which only the room spread reaches.
    #[cfg(feature = "native")]
    #[test]
    fn a_room_routed_orbit_still_ducks() {
        fn room_energy(sidechain_loud: bool) -> f32 {
            let mut engine = Engine::new(EngineConfig::native(48_000.0, 8));
            if sidechain_loud {
                engine.evaluate("sound/sine/orbit/0/freq/60/gain/1");
            }
            // superpan with no pan dry, plus an FX send, is what latches the room.
            // The threshold is set well under the bus level so the detector
            // actually engages on this test signal.
            engine.evaluate(
                "sound/sine/orbit/1/freq/440/gain/1/superpan/0.5/verb/0.4\
                 /comp/1/comporbit/0/compthresh/-40/compratio/8",
            );
            render_blocks(&mut engine, 0.4);
            let mut out = vec![0.0f32; engine.host_buffer_size() * 8];
            engine.process_block(&mut out, &[], &[]);
            out.chunks_exact(8)
                .map(|f| f[4..8].iter().map(|s| s.abs()).sum::<f32>())
                .sum()
        }

        let open = room_energy(false);
        assert!(open > 0.0, "the room spread must reach the far channels");
        let ducked = room_energy(true);
        assert!(
            ducked < open * 0.9,
            "a room-routed orbit's compressor must still duck: open {open}, ducked {ducked}"
        );
    }

    #[cfg(feature = "native")]
    #[test]
    fn bass_mono_centres_a_panned_low_tone() {
        // A hard-left sub tone: with the crossover on, the right channel has to
        // carry it too, which is the whole point of the stage.
        fn right_energy(bass_mono: f32) -> f32 {
            let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));
            engine.set_bass_mono(bass_mono);
            engine.evaluate("sound/sine/freq/40/gain/1/pan/0");
            render_blocks(&mut engine, 0.3);
            let mut out = vec![0.0f32; engine.host_buffer_size() * 2];
            engine.process_block(&mut out, &[], &[]);
            out.chunks_exact(2).map(|f| f[1].abs()).sum()
        }

        let off = right_energy(0.0);
        let on = right_energy(200.0);
        assert!(
            on > off * 4.0,
            "bass mono must move a panned sub to the centre: off {off}, on {on}"
        );
    }
}
