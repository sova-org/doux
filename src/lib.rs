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
#[cfg(feature = "native")]
pub mod telemetry;
#[cfg(feature = "native")]
pub mod time;
pub mod types;
pub mod voice;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub enum AudioCmd {
    Evaluate { path: String, tick: Option<u64> },
    Hush,
    Panic,
}

use dsp::{fast_tanh_f32, init_envelope};
use event::Event;

use orbit::Orbit;

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
    MAX_ORBITS,
};
use voice::modulation::ParamId;
use voice::{modulation, Voice, VoiceParams};

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
    loop_start: f32,
    loop_end: f32,
    looping: bool,
    attenuation: f32,
    pan: f32,
    filter_fc: f32,
    filter_q: f32,
    scale_tuning: f32,
    delay: f32,
    hold: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

// Master soft-clip: plain tanh. Identity slope at origin, monotonic, bounded by ±1.
// Loses ~2.4 dB at unity input — the musical price of analog-style saturation.
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
    /// Audio device callback size in samples. Sets pre-allocated buffers.
    pub buffer_size: usize,
    /// Inner DSP block size in samples. Clamped to `[1, MAX_BLOCK]` at
    /// construction; finer-grained value yields lower latency for sample-rate
    /// scheduling at the cost of throughput.
    pub dsp_block_size: usize,
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
            buffer_size: DEFAULT_BUFFER_SIZE,
            dsp_block_size: DEFAULT_DSP_BLOCK_SIZE,
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
            buffer_size: WASM_BUFFER_SIZE,
            dsp_block_size: DEFAULT_DSP_BLOCK_SIZE,
        }
    }
}

pub struct Engine {
    pub(crate) sr: f32,
    pub(crate) isr: f32,
    pub(crate) max_voices: usize,
    pub(crate) voices: Vec<Voice>,
    pub(crate) active_voices: usize,
    pub(crate) orbits: [Orbit; MAX_ORBITS],
    pub(crate) schedule: Schedule,
    pub(crate) time: f64,
    pub(crate) tick: u64,
    pub(crate) output_channels: usize,
    pub(crate) buffer_size: usize,
    /// Inner DSP block size; sized scratch buffers guarantee `.get() ≤ MAX_BLOCK`.
    pub(crate) dsp_block_size: DspBlockSize,
    pub(crate) output: Vec<f32>,
    #[cfg(not(feature = "native"))]
    pub(crate) sample_pool: SamplePool,
    #[cfg(not(feature = "native"))]
    pub(crate) samples: Vec<SampleInfo>,
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
    pub(crate) gm_bank: Option<soundfont::GmBank>,
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
        dsp::fft::init_twiddles();

        let orbits: [Orbit; MAX_ORBITS] = std::array::from_fn(|_| Orbit::new(config.sample_rate));

        #[cfg(feature = "native")]
        let (sample_registry, sample_loader) = {
            let registry = config
                .sample_registry
                .unwrap_or_else(|| Arc::new(SampleRegistry::new()));
            let loader = SampleLoader::new(Arc::clone(&registry));
            (registry, loader)
        };

        Self {
            sr: config.sample_rate,
            isr: 1.0 / config.sample_rate,
            max_voices: config.max_voices,
            voices: vec![Voice::default(); config.max_voices],
            active_voices: 0,
            orbits,
            schedule: Schedule::new(),
            time: 0.0,
            tick: 0,
            output_channels: config.output_channels,
            buffer_size: config.buffer_size,
            dsp_block_size: DspBlockSize::new(config.dsp_block_size),
            output: vec![0.0; config.buffer_size * config.output_channels],
            #[cfg(not(feature = "native"))]
            sample_pool: SamplePool::new(),
            #[cfg(not(feature = "native"))]
            samples: Vec::with_capacity(256),
            sample_index: Vec::new(),
            #[cfg(feature = "native")]
            sample_registry,
            #[cfg(feature = "native")]
            sample_loader,
            #[cfg(feature = "native")]
            recorder: Recorder::new(config.sample_rate),
            #[cfg(feature = "native")]
            orbit_rec_bus: vec![0.0; MAX_ORBITS * config.buffer_size * CHANNELS],
            #[cfg(feature = "native")]
            metrics: config.metrics,
            #[cfg(feature = "soundfont")]
            gm_bank: None,
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

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn dsp_block_size(&self) -> usize {
        self.dsp_block_size.get()
    }

    pub fn max_voices(&self) -> usize {
        self.max_voices
    }

    pub fn active_voices(&self) -> usize {
        self.active_voices
    }

    pub fn sample_index(&self) -> &[SampleEntry] {
        &self.sample_index
    }

    pub fn set_sample_index(&mut self, index: Vec<SampleEntry>) {
        self.sample_index = index;
    }

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

    #[cfg(feature = "native")]
    pub fn time_anchor(&self) -> time::TimeAnchor {
        time::TimeAnchor {
            start_unix_micros: self.engine_start_unix_micros,
            sample_rate: self.sr,
        }
    }

    #[cfg(feature = "soundfont")]
    pub fn load_soundfont(&mut self, path: &std::path::Path) -> Result<(), String> {
        let (samples, bank) = soundfont::load_sf2(path, self.sr)?;
        let batch: Vec<_> = samples
            .into_iter()
            .map(|(name, data)| (name, Arc::new(data)))
            .collect();
        self.sample_registry.insert_batch(batch);
        self.gm_bank = Some(bank);
        Ok(())
    }

    /// Install a pre-decoded soundfont (samples already loaded, bank parsed).
    /// Used by callers that decode SF2 off the audio thread.
    #[cfg(feature = "soundfont")]
    pub fn install_soundfont(
        &mut self,
        samples: Vec<(String, Arc<SampleData>)>,
        bank: soundfont::GmBank,
    ) {
        self.sample_registry.insert_batch(samples);
        self.gm_bank = Some(bank);
    }

    #[cfg(feature = "soundfont")]
    pub fn gm_bank(&self) -> Option<&soundfont::GmBank> {
        self.gm_bank.as_ref()
    }

    #[cfg(feature = "soundfont")]
    pub fn take_gm_bank(&mut self) -> Option<soundfont::GmBank> {
        self.gm_bank.take()
    }

    #[cfg(feature = "soundfont")]
    pub fn set_gm_bank(&mut self, bank: soundfont::GmBank) {
        self.gm_bank = Some(bank);
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
    #[cfg(feature = "native")]
    fn lookup_sample_entry(&self, name: &str, n: usize) -> Option<&SampleEntry> {
        let name_bytes = name.as_bytes();
        let name_len = name.len();
        let matches = |e: &SampleEntry| {
            e.name.len() > name_len
                && e.name.as_bytes()[name_len] == b'/'
                && e.name.as_bytes().starts_with(name_bytes)
        };
        let count = self.sample_index.iter().filter(|e| matches(e)).count();
        if count == 0 {
            return None;
        }
        let wrapped_n = n % count;
        self.sample_index
            .iter()
            .find(|e| matches(e) && e.name[name_len + 1..].parse::<usize>().ok() == Some(wrapped_n))
    }

    /// Try to get a sample from the registry, or request background loading.
    #[cfg(feature = "native")]
    fn get_registry_sample(&mut self, name: &str, n: usize) -> Option<(Arc<str>, Arc<SampleData>)> {
        let (sample_name, path) = {
            let entry = self.lookup_sample_entry(name, n)?;
            (Arc::clone(&entry.name), Arc::clone(&entry.path))
        };

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

    /// Resolve a GM soundfont zone: extract program from sound string, look up zone, get sample.
    #[cfg(feature = "soundfont")]
    fn resolve_gm(&self, event: &Event) -> Option<GmResolved> {
        let sound_str = event.sound.as_ref()?;
        let program_str = sound_str.strip_prefix("gm")?;
        let program_str = if program_str.is_empty() {
            "0"
        } else {
            program_str
        };

        let note = event
            .freq
            .map(|f| (types::freq2midi(f).round() as i32).clamp(0, 127) as u8)
            .unwrap_or(60);
        let vel = (event.velocity.unwrap_or(1.0) * 127.0).clamp(1.0, 127.0) as u8;

        let bank = self.gm_bank.as_ref()?;
        let zone = bank.find(program_str, note, vel)?;
        let data = self.sample_registry.get(zone.sample_name)?;
        Some(GmResolved {
            data,
            root_freq: zone.root_freq,
            loop_start: zone.loop_start,
            loop_end: zone.loop_end,
            looping: zone.looping,
            attenuation: zone.attenuation,
            pan: zone.pan,
            filter_fc: zone.filter_fc,
            filter_q: zone.filter_q,
            scale_tuning: zone.scale_tuning,
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
    /// `play` events are RT-safe: sample note-on now reuses pre-owned metadata and
    /// only clones `Arc` handles on the callback path. `rec` remains non-RT.
    pub fn dispatch_event(&mut self, event: Event) -> Option<usize> {
        let cmd = event.cmd.as_deref().unwrap_or("play");

        match cmd {
            "play" => self.play_event(event),
            #[cfg(feature = "native")]
            "rec" => {
                self.handle_rec(&event);
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
                if let Some(v) = event.voice {
                    if v < self.active_voices {
                        self.voices[v].force_release();
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

    // NOTE: handle_rec allocates (format!, push, insert) but only fires on recording
    // toggle-off, not per-block. Acceptable for now; defer to worker thread if needed.
    #[cfg(feature = "native")]
    fn handle_rec(&mut self, event: &Event) {
        let overdub = event.overdub.unwrap_or(false);
        let name = event.sound.as_deref();
        let orbit = event.orbit;

        if self
            .recorder
            .toggle(name, overdub, orbit, &self.sample_registry)
            .is_some()
        {
            if let Some((name, data)) = self.recorder.finalize() {
                let key = format!("{name}/0");
                self.sample_registry.insert(key.clone(), data);
                if !self.sample_index.iter().any(|e| e.name.as_ref() == key) {
                    self.sample_index.push(SampleEntry {
                        name: Arc::from(key),
                        path: Arc::new(std::path::PathBuf::new()),
                    });
                }
            }
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

        let (voice_idx, is_new_voice) = if let Some(reuse_idx) = cut_reuse {
            (reuse_idx, true)
        } else if let Some(v) = event.voice {
            if v < self.active_voices {
                // Voice exists - reuse it
                (v, false)
            } else {
                // Voice index out of range - allocate new
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
                (i, true)
            }
        } else {
            // No voice specified - allocate new
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
            (i, true)
        };

        let should_reset = is_new_voice || event.reset.unwrap_or(false);

        if should_reset {
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

        // Update voice params (only the ones explicitly set in event)
        self.update_voice_params(voice_idx, event);
        self.voices[voice_idx].ensure_effects();

        Some(voice_idx)
    }

    /// Update voice params - only updates fields that are explicitly set in the event
    fn update_voice_params(&mut self, idx: usize, event: &Event) {
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
            set_pos!(delay, orbit.delay_level);
            set_pos!(verb, orbit.verb_level);
            set_pos!(comb, orbit.comb_level);
            set_pos!(feedback, orbit.fb_level);
            set_pos!(comp, orbit.comp.params.amount);
            set!(delaytime, orbit.delay.params.time);
            set!(delayfeedback, orbit.delay.params.feedback);
            set!(delaytype, orbit.delay.params.delay_type);
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
            set!(verbchorus, orbit.reverb_params.chorus);
            set!(verbchorusfreq, orbit.reverb_params.chorus_freq);
            set!(combfreq, orbit.comb_params.freq);
            set!(combfeedback, orbit.comb_params.feedback);
            set!(combdamp, orbit.comb_params.damp);
            set!(fbtime, orbit.fb.params.time_ms);
            set!(fbdamp, orbit.fb.params.damp);
            set!(fbcross, orbit.fb.params.cross);
            set!(fblfo, orbit.fb.params.lfo);
            set!(fblfodepth, orbit.fb.params.lfo_depth);
            set!(fblfoshape, orbit.fb.params.lfo_shape);
            set!(compattack, orbit.comp.params.attack);
            set!(comprelease, orbit.comp.params.release);
            set!(comporbit, orbit.comp_orbit);
        }

        let v = &mut self.voices[idx];

        // --- Pitch ---
        copy_opt!(event, v.params, freq, detune, speed);
        if let Some(stretch) = event.stretch {
            v.params.stretch = stretch.max(0.0);
        }
        // --- Source ---
        if let Some(source) = parsed_source {
            v.params.sound = source;
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
        if let Some(partials) = event.partials {
            v.params.partials = partials.clamp(1.0, 32.0);
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
        if let Some(gm) = gm_resolved {
            let mut rs = RegistrySample::new(None, gm.data, 0.0, 1.0);
            rs.root_freq = gm.root_freq;
            rs.scale_tuning = gm.scale_tuning;
            if gm.looping {
                rs.set_loop(gm.loop_start, gm.loop_end);
            }
            rs.attenuation = gm.attenuation;
            v.registry_sample = Some(rs);
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
            if event.lpf.is_none() && gm.filter_fc < 19500.0 {
                v.params.lpf = Some(gm.filter_fc);
                v.params.lpq = gm.filter_q;
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

        // --- Gain Envelope ---
        let (att, dec, sus, rel) =
            if let Some((d_freq, d_att, d_dec, d_sus, d_rel)) = v.params.sound.drum_defaults() {
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
        copy_opt!(event, v.params, flanger, flangerdepth, flangerfeedback);
        copy_opt!(event, v.params, smear, smearfreq, smearfb);
        copy_opt!(event, v.params, chorus, chorusdepth, chorusdelay);
        copy_opt_some!(event, v.params, coarse, crush, fold, wrap, distort);
        copy_opt!(event, v.params, distortvol);
        copy_opt!(event, v.params, width, haas);
        copy_opt!(event, v.params, eqlo, eqmid, eqhi, eqlofreq, eqmidfreq, eqhifreq, tilt);

        // --- Routing (orbit FX state lives on the orbit, not the voice) ---
        copy_opt!(event, v.params, orbit);

        // Live input channel
        v.params.inchan = event.inchan;

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
    ///    `(start+f)*output_channels`, then per-pair soft-clip.
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
        #[cfg(feature = "native")]
        let recorder_active = self.recorder.target_orbit().is_some();

        // Step 1: clear orbit buses for this chunk.
        for orbit in &mut self.orbits {
            orbit.clear_bus(n);
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
                let Some((env, freq)) = voice.prepare_block(isr, n) else {
                    Self::free_voice_in(voices, active_voices, i);
                    continue;
                };

                let source_start = Instant::now();
                let w = voice.run_source_block(
                    freq,
                    isr,
                    n,
                    web_pcm,
                    start,
                    live_input,
                    input_channels,
                );
                *voice_source_ns += source_start.elapsed().as_nanos() as u64;

                if w == 0 {
                    Self::free_voice_in(voices, active_voices, i);
                    continue;
                }
                // Zero the tail so accumulation reads clean frames.
                for j in w..n {
                    voice.scratch[j] = [0.0; CHANNELS];
                }

                let fx_start = Instant::now();
                voice.apply_filters_and_effects_block(&env, isr, w);
                *voice_fx_ns += fx_start.elapsed().as_nanos() as u64;
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

            // Accumulate this voice's output into its orbit bus.
            let orbit_idx = voice.params.orbit % MAX_ORBITS;
            let orbit = &mut orbits[orbit_idx];
            for f in 0..written {
                for c in 0..CHANNELS {
                    orbit.bus[f][c] += voice.scratch[f][c];
                }
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

        // Clear all destination slots for this chunk first.
        for f in 0..n {
            let base_idx = (start + f) * output_channels;
            for c in 0..output_channels {
                output[base_idx + c] = 0.0;
            }
        }

        #[cfg(feature = "native")]
        let orbit_rec_bus = &mut self.orbit_rec_bus;

        for f in 0..n {
            let base_idx = (start + f) * output_channels;
            let sample_idx = start + f;

            // Snapshot per-orbit bus values for this frame so the sidechain
            // read sees the post-FX state without re-borrowing.
            let mut frame_bus = [[0.0f32; CHANNELS]; MAX_ORBITS];
            for (oi, orbit) in orbits.iter().enumerate() {
                frame_bus[oi] = orbit.bus[f];
            }

            for (oi, orbit) in orbits.iter_mut().enumerate() {
                let out_pair = oi % num_pairs;
                let pair_offset = out_pair * 2;
                let cp = orbit.comp.params;

                let orbit_frame = frame_bus[oi];

                if cp.amount > 0.0 {
                    let sc = orbit.comp_orbit % MAX_ORBITS;
                    let sc_total = frame_bus[sc];
                    let sc_level = sc_total[0].abs().max(sc_total[1].abs());
                    let attack_coeff = (isr / cp.attack.max(0.0001)).min(1.0);
                    let release_coeff = (isr / cp.release.max(0.0001)).min(1.0);
                    let env = orbit.comp.process(sc_level, attack_coeff, release_coeff);
                    let gain = (1.0 - env).powf(1.0 + cp.amount * 4.0);
                    for c in 0..CHANNELS {
                        output[base_idx + pair_offset + c] += orbit_frame[c] * gain;
                    }
                    #[cfg(feature = "native")]
                    if recorder_active {
                        let bus_idx = (oi * total + sample_idx) * CHANNELS;
                        orbit_rec_bus[bus_idx] = orbit_frame[0] * gain;
                        orbit_rec_bus[bus_idx + 1] = orbit_frame[1] * gain;
                    }
                } else {
                    for c in 0..CHANNELS {
                        output[base_idx + pair_offset + c] += orbit_frame[c];
                    }
                    #[cfg(feature = "native")]
                    if recorder_active {
                        let bus_idx = (oi * total + sample_idx) * CHANNELS;
                        orbit_rec_bus[bus_idx] = orbit_frame[0];
                        orbit_rec_bus[bus_idx + 1] = orbit_frame[1];
                    }
                }
            }

            for pair_index in 0..num_pairs {
                let pair_base = base_idx + pair_index * CHANNELS;
                output[pair_base] = soft_clip_sample(output[pair_base]);
                output[pair_base + 1] = soft_clip_sample(output[pair_base + 1]);
            }
        }

        #[cfg(all(feature = "native", feature = "profiling"))]
        {
            *final_mix_ns += final_mix_start.elapsed().as_nanos() as u64;
        }
    }

    pub fn process_block(&mut self, output: &mut [f32], web_pcm: &[f32], live_input: &[f32]) {
        // Wall-clock for the load gate + `BlockTotal` metric. Permitted on the
        // audio thread per `to_do.md` real-time invariants: resolves via VDSO
        // (`mach_absolute_time` / `clock_gettime(CLOCK_MONOTONIC)`), no kernel
        // transition. Load-bearing for overload-driven voice shedding below.
        #[cfg(feature = "native")]
        let start = std::time::Instant::now();

        let samples = output.len() / self.output_channels;

        #[cfg(feature = "native")]
        {
            // SAFETY: orbit_rec_bus is pre-allocated in constructor to buffer_size capacity.
            // This debug_assert catches mismatches during development without panicking in release.
            let needed = MAX_ORBITS * samples * CHANNELS;
            debug_assert!(
                self.orbit_rec_bus.len() >= needed,
                "orbit_rec_bus too small: {} < {needed}",
                self.orbit_rec_bus.len()
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
        let bs = self.dsp_block_size.get();
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

        // SAFETY: output is pre-allocated in constructor to buffer_size capacity.
        // If output grew (e.g. dynamic block size), just copy what fits.
        let copy_len = output.len().min(self.output.len());
        self.output[..copy_len].copy_from_slice(&output[..copy_len]);
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
}
