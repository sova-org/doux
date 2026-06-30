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
    /// deallocate a `Box` on receive. Note: an immediate event's interior
    /// `String`/`Vec` fields are still freed on the audio thread when it is
    /// dropped after dispatch; scheduled events move into the bounded schedule
    /// and only drop when they fire.
    DispatchEvent(event::Event),
    Hush,
    Panic,
}

use dsp::{fast_tanh_f32, ftz, init_envelope};
use event::Event;

use orbit::Orbit;

/// Re-export so downstream crates (e.g. `doux-sova`) can name the swap
/// type used by [`Engine::sample_index_handle`] without adding `arc-swap`
/// to their own `Cargo.toml`.
#[cfg(feature = "native")]
pub use arc_swap;
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
#[cfg(feature = "native")]
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

// Master safety clip, after the limiter: plain tanh. Identity slope at origin,
// monotonic, bounded by ±1. The limiter holds peaks near LIMITER_THRESHOLD,
// so program material passes this at near-unity slope; only attack edges the
// limiter's instant-attack envelope hasn't caught yet get rounded.
#[inline]
fn soft_clip_sample(input: f32) -> f32 {
    fast_tanh_f32(input)
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
    pub(crate) output_channels: usize,
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
    /// Master DC-blocker one-pole HP state, one slot per output channel.
    /// Persists across chunks — `gen_block` must never reset it.
    master_dc: [f32; MAX_OUTPUT_CHANNELS],
    /// One-pole coeff for the `MASTER_DC_HZ` high-pass (sr is fixed for the
    /// engine's lifetime, same invariant as `isr`).
    master_dc_coeff: f32,
    /// Master linked peak limiter (state persists across chunks).
    limiter: effects::Limiter,
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
}

#[cfg(feature = "native")]
fn now_unix_micros() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
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
            output_channels: config.output_channels,
            host_buffer_size: config.host_buffer_size,
            inner_block_size: DspBlockSize::new(config.inner_block_size),
            output: vec![0.0; config.host_buffer_size * config.output_channels],
            superpan_acc: vec![0.0; MAX_BLOCK * config.output_channels],
            superpan_acc_used: false,
            master_dc: [0.0; MAX_OUTPUT_CHANNELS],
            // Bilinear one-pole coeff: w = PI * f / sr, coeff = 2w / (1 + 2w).
            master_dc_coeff: {
                let w = std::f32::consts::PI * MASTER_DC_HZ / config.sample_rate;
                (2.0 * w) / (1.0 + 2.0 * w)
            },
            limiter: effects::Limiter::default(),
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
                None
            }
            "panic" => {
                self.panic();
                None
            }
            "reset" => {
                self.panic();
                self.schedule.clear();
                self.time = 0.0;
                self.tick = 0;
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
                None
            }
            "hush_endless" => {
                for i in 0..self.active_voices {
                    if self.voices[i].params.gate == 0.0 {
                        self.voices[i].force_release();
                    }
                }
                None
            }
            "reset_time" => {
                self.time = 0.0;
                self.tick = 0;
                None
            }
            "reset_schedule" => {
                self.schedule.clear();
                None
            }
            _ => None,
        }
    }

    fn play_event(&mut self, mut event: Event) -> Option<usize> {
        if let Some(delta) = event.delta {
            let base = event.tick.unwrap_or(self.tick) as i64;
            event.tick = Some((base + delta).max(0) as u64);
            event.delta = None;
        }
        if event.tick.is_some() {
            self.schedule.push(event);
            return None;
        }
        self.process_event(&event)
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
            return;
        }
        if let Some(name) = event.sound.take() {
            self.recorder
                .start(name, event.overdub.unwrap_or(false), event.orbit);
        }
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

        // `voice/N` is an identity tag: scan the sounding voices for it.
        let tagged = event
            .voice
            .and_then(|tag| (0..self.active_voices).find(|&i| self.voices[i].tag == Some(tag)));
        let has_sound = event.sound.is_some() || has_web_sample;

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
            // Allocate new
            #[cfg(feature = "native")]
            if self.load_gate || self.active_voices >= self.max_voices {
                return None;
            }
            #[cfg(not(feature = "native"))]
            if self.active_voices >= self.max_voices {
                return None;
            }
            let i = self.active_voices;
            self.active_voices += 1;
            (i, EventMode::New)
        };

        if mode == EventMode::New {
            let old_env = if cut_reuse.is_some() {
                self.voices[voice_idx].dahdsr.current_val
            } else {
                0.0
            };
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
        // Resolve sound/sample first (before borrowing voice)
        // If sound parses as a Source, use it; otherwise treat as sample folder name
        #[cfg(feature = "native")]
        let (registry_sample_data, registry_sample_data_b, sample_blend) =
            if let Some(ref sound_str) = event.sound {
                if sound_str.parse::<Source>().is_ok() {
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
            if sound_str.parse::<Source>().is_err() {
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
            // Statics displace any active ModChain on the same orbit param,
            // mirroring the voice-param semantics below.
            for &id in &event.orbit_static_ids {
                orbit.clear_mod(id);
            }
            set_pos!(delay, orbit.delay_level);
            set_pos!(verb, orbit.verb_level);
            set_pos!(comb, orbit.comb_level);
            set_pos!(feedback, orbit.fb_level);
            set_pos!(comp, orbit.comp.params.amount);
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
            set!(comporbit, orbit.comp_orbit);
            // Inline mods install last (an event carrying both a static and a
            // chain on the same param keeps the chain). Envelope chains
            // trigger on install; the event gate sets their release point
            // (0.0 = hold at sustain).
            let mod_gate = event.gate.unwrap_or(0.0);
            for &(id, chain) in &event.orbit_mods {
                orbit.set_mod(id, chain, mod_gate);
            }
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
                v.params.speed = sample_dur / target_dur;
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
                    v.params.speed = sample_dur / target_dur;
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
        copy_opt!(event, v.params, fm, fmh, fmshape, fm2, fm2h, fmpivot, fmfb);
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
        copy_opt!(event, v.params, flanger, flangerdepth, flangerfeedback, flangermode);
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
        copy_opt!(event, v.params, distortvol, distortmode, distortasym, foldmode);
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
        }
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
        let input_channels = self.input_channels;
        let output_channels = self.output_channels;
        let master_dc = &mut self.master_dc;
        let master_dc_coeff = self.master_dc_coeff;
        let limiter = &mut self.limiter;
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
        #[cfg(not(feature = "native"))]
        let pool = self.sample_pool.data.as_slice();
        #[cfg(not(feature = "native"))]
        let samples_slice = self.samples.as_slice();

        let mut i = 0;
        while i < *active_voices {
            let voice = &mut voices[i];

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
                let mut gains = [0.0f32; superpan::MAX_SUPERPAN_NODES];
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
                // Voice died mid-block; swap last active into slot `i` and
                // re-check the new occupant.
                Self::free_voice_in(voices, active_voices, i);
                continue;
            }

            i += 1;
        }

        // Step 3: orbit FX chain — block-rate.
        #[cfg(all(feature = "native", feature = "profiling"))]
        let orbit_fx_start = std::time::Instant::now();
        for orbit in orbits.iter_mut() {
            orbit.process_block(n);
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
        // Equal-power normalization for diffuse room-wet spread across N channels.
        let room_gain = 1.0 / (output_channels as f32).sqrt();

        // Orbit-major passes. Each output slot still receives its contributions
        // in the same order as the old frame-major loop — orbits ascending, then
        // superpan, then room wet, then soft-clip — so the f32 sums are
        // bit-identical while everything block-constant (compressor coeffs,
        // routing flags) is hoisted out of the per-frame work.

        // Pass 0: clear this chunk's destination slots (contiguous region).
        output[start * output_channels..(start + n) * output_channels].fill(0.0);

        #[cfg(feature = "native")]
        let orbit_rec_bus = &mut self.orbit_rec_bus;

        // Pass 1: non-room orbits onto their stereo pair. Sidechain levels are
        // staged through a stack scratch so reading another orbit's post-FX bus
        // doesn't alias the mutable compressor borrow.
        let mut sc_lv = [0.0f32; MAX_BLOCK];
        for oi in 0..MAX_ORBITS {
            // Room-routed orbits contribute only their wet, spread below — skip
            // the stereo-pair mapping, compressor, and recorder for them.
            if orbits[oi].room_active {
                continue;
            }
            let pair_offset = (oi % num_pairs) * 2;
            let cp = orbits[oi].comp.params;

            // Idle orbit: provably all-zero bus contributes nothing. Cannot
            // skip when the compressor is engaged (its envelope must keep
            // following the sidechain through silence) or when the recorder
            // captures this orbit (its rows must be written every frame).
            #[cfg(feature = "native")]
            let rec_this = rec_orbit == Some(oi);
            #[cfg(not(feature = "native"))]
            let rec_this = false;
            if !orbits[oi].bus_used && cp.amount == 0.0 && !rec_this {
                continue;
            }

            if cp.amount > 0.0 {
                let sc = orbits[oi].comp_orbit % MAX_ORBITS;
                let attack_coeff = (isr / cp.attack.max(0.0001)).min(1.0);
                let release_coeff = (isr / cp.release.max(0.0001)).min(1.0);
                let expo = 1.0 + cp.amount * 4.0;
                for (slot, frame) in sc_lv.iter_mut().zip(orbits[sc].bus.iter()).take(n) {
                    *slot = frame[0].abs().max(frame[1].abs());
                }
                let orbit = &mut orbits[oi];
                for (f, &sc_level) in sc_lv.iter().enumerate().take(n) {
                    let env = orbit.comp.process(sc_level, attack_coeff, release_coeff);
                    let base = 1.0 - env;
                    // IEEE 754: powf(1.0, y) == 1.0 exactly, so the skip is free.
                    let gain = if base == 1.0 { 1.0 } else { base.powf(expo) };
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

        // Pass 3: spread each room-routed orbit's FX wet diffusely across the room.
        for orbit in orbits.iter() {
            if !orbit.room_active {
                continue;
            }
            for f in 0..n {
                let w0 = orbit.fx_wet[f][0];
                let w1 = orbit.fx_wet[f][1];
                let base_idx = (start + f) * output_channels;
                for c in 0..output_channels {
                    let s = if c % 2 == 0 { w0 } else { w1 };
                    output[base_idx + c] += s * room_gain;
                }
            }
        }

        // Pass 4: master chain — per-channel DC blocker (~10 Hz one-pole HP),
        // linked peak limiter (instant attack, ~100 ms release), then tanh
        // safety clip. The limiter computes ONE gain per frame from the peak
        // across all channels so the multichannel image never shifts. All
        // state lives on the Engine: gen_block runs once per inner chunk and
        // must not reset it.
        let limiter_release = (isr / effects::LIMITER_RELEASE_SECS).min(1.0);
        for f in 0..n {
            let base_idx = (start + f) * output_channels;
            let frame = &mut output[base_idx..base_idx + output_channels];
            let mut peak = 0.0_f32;
            for (c, s) in frame.iter_mut().enumerate() {
                // One-pole HP: track the low band in state, subtract. DC is
                // removed before peak detection so offset doesn't eat limiter
                // headroom or bias the tanh asymmetrically.
                let st = &mut master_dc[c];
                *st += master_dc_coeff * (*s - *st);
                let y = *s - *st;
                *s = y;
                peak = peak.max(y.abs());
            }
            let gain = limiter.process(peak, limiter_release);
            for s in frame.iter_mut() {
                *s = soft_clip_sample(*s * gain);
            }
        }
        // Denormal hygiene for non-FTZ targets (wasm): flush decayed DC state
        // once per chunk.
        for st in master_dc.iter_mut().take(output_channels) {
            *st = ftz(*st, 1.0e-12);
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
                self.time = self.tick as f64 / self.sr as f64;
            }

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

    pub fn hush(&mut self) {
        for i in 0..self.active_voices {
            self.voices[i].force_release();
        }
    }

    pub fn panic(&mut self) {
        self.active_voices = 0;
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
}
