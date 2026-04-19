#[cfg(feature = "native")]
pub mod audio;
#[cfg(feature = "native")]
pub mod benchmark;
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
pub mod types;
pub mod voice;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub enum AudioCmd {
    Evaluate(String),
    Hush,
    Panic,
}

use dsp::{fast_tanh_f32, init_envelope};
use event::Event;

use orbit::Orbit;
use types::ModuleInfo;

/// All modules in the engine: sources, effects, filters, modulation.
pub fn all_modules() -> Vec<&'static ModuleInfo> {
    let mut modules: Vec<&'static ModuleInfo> =
        Source::all().iter().map(|s| &s.info().module).collect();
    modules.extend_from_slice(effects::ALL_MODULES);
    modules
}
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
use types::DEFAULT_NATIVE_BLOCK_SIZE;
#[cfg(not(feature = "native"))]
use types::WASM_BLOCK_SIZE;
use types::{Source, CHANNELS, DEFAULT_MAX_VOICES, MAX_ORBITS};
use voice::modulation::ParamId;
use voice::{modulation, Voice, VoiceParams};

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

#[derive(Clone, Copy)]
struct OrbitBlockState {
    params: orbit::EffectParams,
    fb_level: f32,
    has_delay: bool,
    has_verb: bool,
    has_comb: bool,
    has_feedback: bool,
    has_comp: bool,
}

impl Default for OrbitBlockState {
    fn default() -> Self {
        Self {
            params: orbit::EffectParams::default(),
            fb_level: 0.0,
            has_delay: false,
            has_verb: false,
            has_comb: false,
            has_feedback: false,
            has_comp: false,
        }
    }
}

const MIX_BUS_TRIM: f32 = 0.9;
const OUTPUT_LIMIT_THRESHOLD: f32 = 0.95;
const OUTPUT_LIMIT_RELEASE_SECS: f32 = 0.05;
const OUTPUT_SOFT_CLIP_THRESHOLD: f32 = 0.95;

#[derive(Clone, Copy)]
struct StereoOutputStage {
    gain: f32,
}

impl Default for StereoOutputStage {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl StereoOutputStage {
    fn process(&mut self, pair: &mut [f32], sr: f32) {
        debug_assert_eq!(pair.len(), CHANNELS);

        pair[0] *= MIX_BUS_TRIM;
        pair[1] *= MIX_BUS_TRIM;

        let peak = pair[0].abs().max(pair[1].abs());
        let target_gain = if peak > OUTPUT_LIMIT_THRESHOLD {
            OUTPUT_LIMIT_THRESHOLD / peak
        } else {
            1.0
        };

        if target_gain < self.gain {
            self.gain = target_gain;
        } else {
            let release_coeff = 1.0 / (OUTPUT_LIMIT_RELEASE_SECS * sr).max(1.0);
            self.gain += release_coeff * (target_gain - self.gain);
        }

        pair[0] = soft_clip_sample(pair[0] * self.gain);
        pair[1] = soft_clip_sample(pair[1] * self.gain);
    }
}

#[inline]
fn soft_clip_sample(input: f32) -> f32 {
    let magnitude = input.abs();
    if magnitude <= OUTPUT_SOFT_CLIP_THRESHOLD {
        return input;
    }

    let sign = input.signum();
    let headroom = 1.0 - OUTPUT_SOFT_CLIP_THRESHOLD;
    let drive = (magnitude - OUTPUT_SOFT_CLIP_THRESHOLD) / headroom;
    let clipped = OUTPUT_SOFT_CLIP_THRESHOLD + fast_tanh_f32(drive) * headroom;
    sign * clipped.min(1.0)
}

fn output_stage_count(output_channels: usize) -> usize {
    debug_assert_eq!(
        output_channels % CHANNELS,
        0,
        "output channels must be arranged as stereo pairs"
    );
    output_channels / CHANNELS
}

pub struct Engine {
    pub sr: f32,
    pub isr: f32,
    pub max_voices: usize,
    pub voices: Vec<Voice>,
    pub active_voices: usize,
    pub orbits: Vec<Orbit>,
    pub schedule: Schedule,
    pub time: f64,
    pub tick: u64,
    pub output_channels: usize,
    pub block_size: usize,
    pub output: Vec<f32>,
    // Sample storage (WASM only)
    #[cfg(not(feature = "native"))]
    pub sample_pool: SamplePool,
    #[cfg(not(feature = "native"))]
    pub samples: Vec<SampleInfo>,
    // Sample index (native uses registry, WASM uses pool)
    pub sample_index: Vec<SampleEntry>,
    // Lock-free sample registry (native only)
    #[cfg(feature = "native")]
    pub sample_registry: Arc<SampleRegistry>,
    #[cfg(feature = "native")]
    pub sample_loader: SampleLoader,
    #[cfg(feature = "native")]
    recorder: Recorder,
    #[cfg(feature = "native")]
    orbit_rec_bus: Vec<f32>,
    // Telemetry (native only)
    #[cfg(feature = "native")]
    pub metrics: Arc<EngineMetrics>,
    #[cfg(feature = "soundfont")]
    pub gm_bank: Option<soundfont::GmBank>,
    pub input_channels: usize,
    output_stages: Vec<StereoOutputStage>,
    voice_seed: u32,
    #[cfg(feature = "native")]
    load_gate: bool,
}

impl Engine {
    #[cfg(not(feature = "native"))]
    pub fn new(sample_rate: f32) -> Self {
        Self::new_with_channels(sample_rate, CHANNELS, DEFAULT_MAX_VOICES)
    }

    #[cfg(not(feature = "native"))]
    pub fn new_with_channels(sample_rate: f32, output_channels: usize, max_voices: usize) -> Self {
        dsp::fft::init_twiddles();

        let mut orbits = Vec::with_capacity(MAX_ORBITS);
        for _ in 0..MAX_ORBITS {
            orbits.push(Orbit::new(sample_rate));
        }

        Self {
            sr: sample_rate,
            isr: 1.0 / sample_rate,
            max_voices,
            voices: vec![Voice::default(); max_voices],
            active_voices: 0,
            orbits,
            schedule: Schedule::new(),
            time: 0.0,
            tick: 0,
            output_channels,
            block_size: WASM_BLOCK_SIZE,
            output: vec![0.0; WASM_BLOCK_SIZE * output_channels],
            sample_pool: SamplePool::new(),
            samples: Vec::with_capacity(256),
            sample_index: Vec::new(),
            input_channels: 2,
            output_stages: vec![StereoOutputStage::default(); output_stage_count(output_channels)],
            voice_seed: 123456789,
        }
    }

    #[cfg(feature = "native")]
    pub fn new(sample_rate: f32) -> Self {
        Self::new_with_channels(
            sample_rate,
            CHANNELS,
            DEFAULT_MAX_VOICES,
            DEFAULT_NATIVE_BLOCK_SIZE,
        )
    }

    #[cfg(feature = "native")]
    pub fn new_with_channels(
        sample_rate: f32,
        output_channels: usize,
        max_voices: usize,
        block_size: usize,
    ) -> Self {
        dsp::fft::init_twiddles();

        let registry = Arc::new(SampleRegistry::new());
        let loader = SampleLoader::new(Arc::clone(&registry));

        let mut orbits = Vec::with_capacity(MAX_ORBITS);
        for _ in 0..MAX_ORBITS {
            orbits.push(Orbit::new(sample_rate));
        }

        Self {
            sr: sample_rate,
            isr: 1.0 / sample_rate,
            max_voices,
            voices: vec![Voice::default(); max_voices],
            active_voices: 0,
            orbits,
            schedule: Schedule::new(),
            time: 0.0,
            tick: 0,
            output_channels,
            block_size,
            output: vec![0.0; block_size * output_channels],
            sample_index: Vec::new(),
            sample_registry: registry,
            sample_loader: loader,
            recorder: Recorder::new(sample_rate),
            orbit_rec_bus: vec![0.0; MAX_ORBITS * block_size * CHANNELS],
            metrics: Arc::new(EngineMetrics::default()),
            #[cfg(feature = "soundfont")]
            gm_bank: None,
            input_channels: 2,
            output_stages: vec![StereoOutputStage::default(); output_stage_count(output_channels)],
            voice_seed: 123456789,
            load_gate: false,
        }
    }

    #[cfg(feature = "native")]
    pub fn new_with_metrics(
        sample_rate: f32,
        output_channels: usize,
        max_voices: usize,
        metrics: Arc<EngineMetrics>,
        block_size: usize,
    ) -> Self {
        dsp::fft::init_twiddles();

        let registry = Arc::new(SampleRegistry::new());
        let loader = SampleLoader::new(Arc::clone(&registry));

        let mut orbits = Vec::with_capacity(MAX_ORBITS);
        for _ in 0..MAX_ORBITS {
            orbits.push(Orbit::new(sample_rate));
        }

        Self {
            sr: sample_rate,
            isr: 1.0 / sample_rate,
            max_voices,
            voices: vec![Voice::default(); max_voices],
            active_voices: 0,
            orbits,
            schedule: Schedule::new(),
            time: 0.0,
            tick: 0,
            output_channels,
            block_size,
            output: vec![0.0; block_size * output_channels],
            sample_index: Vec::new(),
            sample_registry: registry,
            sample_loader: loader,
            recorder: Recorder::new(sample_rate),
            orbit_rec_bus: vec![0.0; MAX_ORBITS * block_size * CHANNELS],
            metrics,
            #[cfg(feature = "soundfont")]
            gm_bank: None,
            input_channels: 2,
            output_stages: vec![StereoOutputStage::default(); output_stage_count(output_channels)],
            voice_seed: 123456789,
            load_gate: false,
        }
    }

    #[cfg(feature = "soundfont")]
    pub fn load_soundfont(&mut self, path: &std::path::Path) -> Result<(), String> {
        let (samples, bank) = soundfont::load_sf2(path, self.sr)?;
        let presets = bank.preset_count();
        let sample_count = samples.len();
        let batch: Vec<_> = samples
            .into_iter()
            .map(|(name, data)| (name, Arc::new(data)))
            .collect();
        self.sample_registry.insert_batch(batch);
        self.gm_bank = Some(bank);
        println!("SF2: {sample_count} samples, {presets} presets");
        Ok(())
    }

    #[cfg(feature = "soundfont")]
    pub fn load_soundfont_from_dir(&mut self, dir: &std::path::Path) {
        if let Some(sf2_path) = soundfont::find_sf2_file(dir) {
            if let Err(e) = self.load_soundfont(&sf2_path) {
                eprintln!("Failed to load soundfont: {e}");
            }
        }
    }

    #[cfg(not(feature = "native"))]
    pub fn load_sample(&mut self, samples: &[f32], channels: u8, freq: f32) -> Option<usize> {
        let info = self.sample_pool.add(samples, channels, freq)?;
        let idx = self.samples.len();
        self.samples.push(info);
        Some(idx)
    }

    /// Look up sample by name (e.g., "wave_tek") and n (e.g., 0 for "wave_tek/0")
    /// n wraps around using modulo if it exceeds the folder count
    #[cfg(feature = "native")]
    fn find_sample_index(&self, name: &str, n: usize) -> Option<usize> {
        let name_bytes = name.as_bytes();
        let has_prefix = |e: &&SampleEntry| {
            e.name.len() > name.len()
                && e.name.as_bytes()[name_bytes.len()] == b'/'
                && e.name.as_bytes().starts_with(name_bytes)
        };
        let count = self.sample_index.iter().filter(has_prefix).count();
        if count == 0 {
            return None;
        }
        let wrapped_n = n % count;
        self.sample_index.iter().position(|e| {
            e.name.len() > name.len()
                && e.name.as_bytes()[name_bytes.len()] == b'/'
                && e.name.as_bytes().starts_with(name_bytes)
                && e.name[name.len() + 1..].parse::<usize>().ok() == Some(wrapped_n)
        })
    }

    /// Get the sample name for a given base name and n index.
    #[cfg(feature = "native")]
    fn get_sample_name(&self, name: &str, n: usize) -> Option<Arc<str>> {
        let index_idx = self.find_sample_index(name, n)?;
        Some(Arc::clone(&self.sample_index[index_idx].name))
    }

    /// Try to get a sample from the registry, or request background loading.
    #[cfg(feature = "native")]
    fn get_registry_sample(&mut self, name: &str, n: usize) -> Option<(Arc<str>, Arc<SampleData>)> {
        let sample_name = self.get_sample_name(name, n)?;

        if let Some(data) = self.sample_registry.get(sample_name.as_ref()) {
            if data.frame_count < data.total_frames {
                let index_idx = self.find_sample_index(name, n)?;
                let path = Arc::clone(&self.sample_index[index_idx].path);
                self.sample_loader
                    .request(Arc::clone(&sample_name), path, self.sr);
            }
            return Some((sample_name, data));
        }

        let index_idx = self.find_sample_index(name, n)?;
        let path = Arc::clone(&self.sample_index[index_idx].path);
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
        copy_opt_some!(event, v.params, llpf);
        copy_opt!(event, v.params, llpq);
        copy_opt_some!(event, v.params, lhpf);
        copy_opt!(event, v.params, lhpq);
        copy_opt_some!(event, v.params, lbpf);
        copy_opt!(event, v.params, lbpq);

        // --- Modulation ---
        copy_opt!(event, v.params, vib, vibmod, vibshape);
        copy_opt!(event, v.params, fm, fmh, fmshape, fm2, fm2h, fmalgo, fmfb);
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
        copy_opt!(event, v.params, comb, combfreq, combfeedback, combdamp);
        copy_opt!(
            event, v.params, feedback, fbtime, fbdamp, fbcross, fblfo, fblfodepth, fblfoshape
        );
        copy_opt!(event, v.params, comp, compattack, comprelease, comporbit);
        copy_opt_some!(event, v.params, coarse, crush, fold, wrap, distort);
        copy_opt!(event, v.params, distortvol);
        copy_opt!(event, v.params, width, haas);
        copy_opt!(event, v.params, eqlo, eqmid, eqhi, eqlofreq, eqmidfreq, eqhifreq, tilt);

        // --- Sends ---
        copy_opt!(
            event,
            v.params,
            orbit,
            delay,
            delaytime,
            delayfeedback,
            delaytype
        );
        copy_opt!(
            event,
            v.params,
            verb,
            verbtype,
            verbdecay,
            verbdamp,
            verbpredelay,
            verbdiff,
            verbprelow,
            verbprehigh,
            verblowcut,
            verbhighcut,
            verblowgain,
            verbchorus,
            verbchorusfreq
        );

        // Live input channel
        v.params.inchan = event.inchan;

        // Install inline parameter modulations
        for (id, chain) in &event.mods {
            v.set_mod(*id, *chain);
        }

        v.sync_source_state();
    }

    fn collect_block_orbit_state(&self) -> [OrbitBlockState; MAX_ORBITS] {
        let mut states = [OrbitBlockState::default(); MAX_ORBITS];
        let num_orbits = self.orbits.len();

        for voice in self.voices.iter().take(self.active_voices) {
            let orbit_idx = voice.params.orbit % num_orbits;
            let state = &mut states[orbit_idx];

            if voice.params.delay > 0.0 {
                state.has_delay = true;
                state.params.delay_time = voice.params.delaytime;
                state.params.delay_feedback = voice.params.delayfeedback;
                state.params.delay_type = voice.params.delaytype;
            }
            if voice.params.verb > 0.0 {
                state.has_verb = true;
                state.params.verb_type = voice.params.verbtype;
                state.params.verb_decay = voice.params.verbdecay;
                state.params.verb_damp = voice.params.verbdamp;
                state.params.verb_predelay = voice.params.verbpredelay;
                state.params.verb_diff = voice.params.verbdiff;
                state.params.verb_prelow = voice.params.verbprelow;
                state.params.verb_prehigh = voice.params.verbprehigh;
                state.params.verb_lowcut = voice.params.verblowcut;
                state.params.verb_highcut = voice.params.verbhighcut;
                state.params.verb_lowgain = voice.params.verblowgain;
                state.params.verb_chorus = voice.params.verbchorus;
                state.params.verb_chorus_freq = voice.params.verbchorusfreq;
            }
            if voice.params.comb > 0.0 {
                state.has_comb = true;
                state.params.comb_freq = voice.params.combfreq;
                state.params.comb_feedback = voice.params.combfeedback;
                state.params.comb_damp = voice.params.combdamp;
            }
            if voice.params.feedback > 0.0 {
                state.has_feedback = true;
                state.fb_level = voice.params.feedback;
                state.params.fb_time = voice.params.fbtime;
                state.params.fb_damp = voice.params.fbdamp;
                state.params.fb_cross = voice.params.fbcross;
                state.params.fb_lfo = voice.params.fblfo;
                state.params.fb_lfo_depth = voice.params.fblfodepth;
                state.params.fb_lfo_shape = voice.params.fblfoshape;
            }
            if voice.params.comp > 0.0 {
                state.has_comp = true;
                state.params.comp = voice.params.comp;
                state.params.comp_attack = voice.params.compattack;
                state.params.comp_release = voice.params.comprelease;
                state.params.comp_orbit = voice.params.comporbit;
            }
        }

        states
    }

    fn apply_block_orbit_state(&mut self, states: &[OrbitBlockState; MAX_ORBITS]) {
        for (orbit, state) in self.orbits.iter_mut().zip(states.iter()) {
            if state.has_delay {
                orbit.params.delay_time = state.params.delay_time;
                orbit.params.delay_feedback = state.params.delay_feedback;
                orbit.params.delay_type = state.params.delay_type;
            }
            if state.has_verb {
                orbit.params.verb_type = state.params.verb_type;
                orbit.params.verb_decay = state.params.verb_decay;
                orbit.params.verb_damp = state.params.verb_damp;
                orbit.params.verb_predelay = state.params.verb_predelay;
                orbit.params.verb_diff = state.params.verb_diff;
                orbit.params.verb_prelow = state.params.verb_prelow;
                orbit.params.verb_prehigh = state.params.verb_prehigh;
                orbit.params.verb_lowcut = state.params.verb_lowcut;
                orbit.params.verb_highcut = state.params.verb_highcut;
                orbit.params.verb_lowgain = state.params.verb_lowgain;
                orbit.params.verb_chorus = state.params.verb_chorus;
                orbit.params.verb_chorus_freq = state.params.verb_chorus_freq;
            }
            if state.has_comb {
                orbit.params.comb_freq = state.params.comb_freq;
                orbit.params.comb_feedback = state.params.comb_feedback;
                orbit.params.comb_damp = state.params.comb_damp;
            }
            if state.has_feedback {
                orbit.params.fb_time = state.params.fb_time;
                orbit.params.fb_damp = state.params.fb_damp;
                orbit.params.fb_cross = state.params.fb_cross;
                orbit.params.fb_lfo = state.params.fb_lfo;
                orbit.params.fb_lfo_depth = state.params.fb_lfo_depth;
                orbit.params.fb_lfo_shape = state.params.fb_lfo_shape;
                orbit.fb_level = state.fb_level;
            } else {
                orbit.fb_level = 0.0;
            }
            if state.has_comp {
                orbit.params.comp = state.params.comp;
                orbit.params.comp_attack = state.params.comp_attack;
                orbit.params.comp_release = state.params.comp_release;
                orbit.params.comp_orbit = state.params.comp_orbit;
            }
        }
    }

    fn free_voice(&mut self, i: usize) {
        if self.active_voices > 0 {
            self.active_voices -= 1;
            self.voices.swap(i, self.active_voices);
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

    #[allow(unused_variables)]
    pub fn gen_sample(
        &mut self,
        output: &mut [f32],
        sample_idx: usize,
        block_samples: usize,
        web_pcm: &[f32],
        live_input: &[f32],
    ) {
        let base_idx = sample_idx * self.output_channels;
        let num_pairs = self.output_channels / 2;

        for c in 0..self.output_channels {
            output[base_idx + c] = 0.0;
        }

        // Clear orbit sends
        for orbit in &mut self.orbits {
            orbit.clear_sends();
        }

        // Process voices - matches dough.c behavior exactly:
        // When a voice dies, it's freed immediately and the loop continues,
        // which means the swapped-in voice (from the end) gets skipped this frame.
        let isr = self.isr;
        let num_orbits = self.orbits.len();
        #[cfg(all(feature = "native", feature = "profiling"))]
        let mut voice_source_ns = 0u64;
        #[cfg(all(feature = "native", feature = "profiling"))]
        let mut voice_fx_ns = 0u64;

        let mut orbit_dry = [[0.0f32; CHANNELS]; MAX_ORBITS];
        let mut i = 0;
        while i < self.active_voices {
            #[cfg(all(feature = "native", feature = "profiling"))]
            let alive = {
                let mut alive = false;
                if let Some((env, freq)) = self.voices[i].prepare_frame(isr) {
                    let source_start = std::time::Instant::now();
                    let source_alive = self.voices[i].run_source(
                        freq,
                        isr,
                        web_pcm,
                        sample_idx,
                        live_input,
                        self.input_channels,
                    );
                    voice_source_ns += source_start.elapsed().as_nanos() as u64;

                    if source_alive {
                        let fx_start = std::time::Instant::now();
                        self.voices[i].apply_filters_and_effects(env, isr);
                        voice_fx_ns += fx_start.elapsed().as_nanos() as u64;
                        alive = true;
                    }
                }
                alive
            };
            #[cfg(all(feature = "native", not(feature = "profiling")))]
            #[cfg(feature = "native")]
            let alive =
                self.voices[i].process(isr, web_pcm, sample_idx, live_input, self.input_channels);
            #[cfg(not(feature = "native"))]
            let alive = {
                let pool = self.sample_pool.data.as_slice();
                let samples = self.samples.as_slice();
                self.voices[i].process(
                    isr,
                    pool,
                    samples,
                    web_pcm,
                    sample_idx,
                    live_input,
                    self.input_channels,
                )
            };
            if !alive {
                self.free_voice(i);
                continue;
            }

            let orbit_idx = self.voices[i].params.orbit % num_orbits;

            orbit_dry[orbit_idx][0] += self.voices[i].ch[0];
            orbit_dry[orbit_idx][1] += self.voices[i].ch[1];

            // Add to orbit sends. Effect params are collected once per block.
            if self.voices[i].params.delay > 0.0 {
                for c in 0..CHANNELS {
                    self.orbits[orbit_idx]
                        .add_delay_send(c, self.voices[i].ch[c] * self.voices[i].params.delay);
                }
            }
            if self.voices[i].params.verb > 0.0 {
                for c in 0..CHANNELS {
                    self.orbits[orbit_idx]
                        .add_verb_send(c, self.voices[i].ch[c] * self.voices[i].params.verb);
                }
            }
            if self.voices[i].params.comb > 0.0 {
                for c in 0..CHANNELS {
                    self.orbits[orbit_idx]
                        .add_comb_send(c, self.voices[i].ch[c] * self.voices[i].params.comb);
                }
            }
            if self.voices[i].params.feedback > 0.0 {
                for c in 0..CHANNELS {
                    self.orbits[orbit_idx]
                        .add_fb_send(c, self.voices[i].ch[c] * self.voices[i].params.feedback);
                }
            }

            i += 1;
        }

        // Phase 1: process all orbits, store output levels
        let mut orbit_levels = [[0.0f32; CHANNELS]; MAX_ORBITS];
        #[cfg(all(feature = "native", feature = "profiling"))]
        let orbit_fx_start = std::time::Instant::now();
        for (oi, orbit) in self.orbits.iter_mut().enumerate() {
            orbit.process();
            orbit_levels[oi] = [
                orbit.delay_out[0] + orbit.verb_out[0] + orbit.comb_out[0] + orbit.fb_out[0],
                orbit.delay_out[1] + orbit.verb_out[1] + orbit.comb_out[1] + orbit.fb_out[1],
            ];
        }
        #[cfg(all(feature = "native", feature = "profiling"))]
        let orbit_fx_ns = orbit_fx_start.elapsed().as_nanos() as u64;

        // Phase 2: mix to output with optional sidechain compression
        let isr = self.isr;
        let num_orbs = self.orbits.len();
        #[cfg(all(feature = "native", feature = "profiling"))]
        let final_mix_start = std::time::Instant::now();
        for (oi, orbit) in self.orbits.iter_mut().enumerate() {
            let out_pair = oi % num_pairs;
            let pair_offset = out_pair * 2;
            let p = &orbit.params;

            let total = [
                orbit_dry[oi][0] + orbit_levels[oi][0],
                orbit_dry[oi][1] + orbit_levels[oi][1],
            ];

            if p.comp > 0.0 {
                let sc = p.comp_orbit % num_orbs;
                let sc_total = [
                    orbit_dry[sc][0] + orbit_levels[sc][0],
                    orbit_dry[sc][1] + orbit_levels[sc][1],
                ];
                let sc_level = sc_total[0].abs().max(sc_total[1].abs());
                let attack_coeff = (isr / p.comp_attack.max(0.0001)).min(1.0);
                let release_coeff = (isr / p.comp_release.max(0.0001)).min(1.0);
                let env = orbit.comp.process(sc_level, attack_coeff, release_coeff);
                let gain = (1.0 - env).powf(1.0 + p.comp * 4.0);
                for c in 0..CHANNELS {
                    output[base_idx + pair_offset + c] += total[c] * gain;
                }
                #[cfg(feature = "native")]
                if self.recorder.target_orbit().is_some() {
                    let bus_idx = (oi * block_samples + sample_idx) * CHANNELS;
                    self.orbit_rec_bus[bus_idx] = total[0] * gain;
                    self.orbit_rec_bus[bus_idx + 1] = total[1] * gain;
                }
            } else {
                for c in 0..CHANNELS {
                    output[base_idx + pair_offset + c] += total[c];
                }
                #[cfg(feature = "native")]
                if self.recorder.target_orbit().is_some() {
                    let bus_idx = (oi * block_samples + sample_idx) * CHANNELS;
                    self.orbit_rec_bus[bus_idx] = total[0];
                    self.orbit_rec_bus[bus_idx + 1] = total[1];
                }
            }
        }

        for (pair_index, stage) in self.output_stages.iter_mut().enumerate().take(num_pairs) {
            let pair_base = base_idx + pair_index * CHANNELS;
            stage.process(&mut output[pair_base..pair_base + CHANNELS], self.sr);
        }

        #[cfg(all(feature = "native", feature = "profiling"))]
        {
            let profiler = &self.metrics.profiler;
            profiler.record_phase(ProfilePhase::VoiceSource, voice_source_ns);
            profiler.record_phase(ProfilePhase::VoiceFx, voice_fx_ns);
            profiler.record_phase(ProfilePhase::OrbitFx, orbit_fx_ns);
            profiler.record_phase(
                ProfilePhase::FinalMix,
                final_mix_start.elapsed().as_nanos() as u64,
            );
        }
    }

    pub fn process_block(&mut self, output: &mut [f32], web_pcm: &[f32], live_input: &[f32]) {
        #[cfg(feature = "native")]
        let start = std::time::Instant::now();

        let samples = output.len() / self.output_channels;

        #[cfg(feature = "native")]
        {
            // SAFETY: orbit_rec_bus is pre-allocated in constructor to block_size capacity.
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
        let mut orbit_state_ready = false;
        for i in 0..samples {
            #[cfg(all(feature = "native", feature = "profiling"))]
            let schedule_start = std::time::Instant::now();
            self.process_schedule();
            #[cfg(all(feature = "native", feature = "profiling"))]
            {
                schedule_elapsed_ns += schedule_start.elapsed().as_nanos() as u64;
            }
            if !orbit_state_ready {
                let orbit_states = self.collect_block_orbit_state();
                self.apply_block_orbit_state(&orbit_states);
                orbit_state_ready = true;
            }
            self.tick += 1;
            self.time = self.tick as f64 / self.sr as f64;
            self.gen_sample(output, i, samples, web_pcm, live_input);
        }
        #[cfg(all(feature = "native", feature = "profiling"))]
        self.metrics
            .profiler
            .record_phase(ProfilePhase::Schedule, schedule_elapsed_ns);

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

        // SAFETY: output is pre-allocated in constructor to block_size capacity.
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

    fn render_delay_send(engine: &mut Engine, blocks: usize) -> [f32; CHANNELS] {
        let mut output = [0.0; CHANNELS];
        for _ in 0..blocks {
            engine.process_block(&mut output, &[], &[]);
        }
        engine.orbits[0].delay_send
    }

    fn test_voice(freq: f32, orbit: usize, delay: f32, delay_time: f32, comp: f32) -> VoiceParams {
        VoiceParams {
            sound: Source::Sine,
            freq,
            gain: 0.3,
            postgain: 0.8,
            gate: 1.0,
            attack: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release: 0.05,
            orbit,
            delay,
            delaytime: delay_time,
            delayfeedback: 0.25 + delay_time,
            comp,
            compattack: 0.01 + delay_time * 0.1,
            comprelease: 0.1 + delay_time * 0.2,
            comporbit: orbit,
            ..VoiceParams::default()
        }
    }

    #[test]
    fn block_orbit_state_uses_last_active_voice_params() {
        let mut engine = Engine::new(48_000.0);
        engine.play(test_voice(220.0, 0, 0.2, 0.15, 0.3));
        engine.play(test_voice(330.0, 0, 0.4, 0.35, 0.6));

        let states = engine.collect_block_orbit_state();

        assert!(states[0].has_delay);
        assert!(states[0].has_comp);
        assert!((states[0].params.delay_time - 0.35).abs() < 1e-6);
        assert!((states[0].params.delay_feedback - 0.6).abs() < 1e-6);
        assert!((states[0].params.comp - 0.6).abs() < 1e-6);
    }

    #[test]
    fn stereo_output_stage_keeps_signal_bounded() {
        let mut stage = StereoOutputStage::default();
        let mut pair = [2.0, -1.5];

        stage.process(&mut pair, 48_000.0);

        assert!(pair[0].abs() <= 1.0);
        assert!(pair[1].abs() <= 1.0);
    }

    #[test]
    fn stereo_output_stage_uses_linked_gain() {
        let mut stage = StereoOutputStage::default();
        let mut pair = [1.2, 0.6];

        stage.process(&mut pair, 48_000.0);

        assert!((pair[0] - 0.95).abs() < 1e-6);
        assert!((pair[1] - 0.475).abs() < 1e-6);
    }

    #[test]
    fn stereo_output_stage_release_recovers_toward_unity() {
        let mut stage = StereoOutputStage::default();
        let mut clipped = [2.0, 2.0];
        stage.process(&mut clipped, 48_000.0);
        let limited_gain = stage.gain;

        let mut quiet = [0.1, 0.1];
        for _ in 0..24_000 {
            stage.process(&mut quiet, 48_000.0);
        }

        assert!(stage.gain > limited_gain);
        assert!(stage.gain > 0.99);
    }

    #[test]
    fn stereo_output_stages_are_independent() {
        let mut stages = [StereoOutputStage::default(); 2];
        let mut loud_pair = [2.0, 2.0];
        let mut quiet_pair = [0.1, 0.1];

        stages[0].process(&mut loud_pair, 48_000.0);
        stages[1].process(&mut quiet_pair, 48_000.0);

        assert!(stages[0].gain < 1.0);
        assert_eq!(stages[1].gain, 1.0);
    }

    #[test]
    fn block_rate_orbit_params_keep_send_sums_without_voice_compensation() {
        let blocks = 512;
        let mut both = Engine::new(48_000.0);
        both.play(test_voice(220.0, 0, 0.2, 0.15, 0.0));
        both.play(test_voice(330.0, 0, 0.4, 0.35, 0.0));
        let both_send = render_delay_send(&mut both, blocks);
        assert!((both.orbits[0].params.delay_time - 0.35).abs() < 1e-6);

        let mut first = Engine::new(48_000.0);
        first.play(test_voice(220.0, 0, 0.2, 0.15, 0.0));
        let first_send = render_delay_send(&mut first, blocks);

        let mut second = Engine::new(48_000.0);
        second.play(test_voice(330.0, 0, 0.4, 0.35, 0.0));
        let second_send = render_delay_send(&mut second, blocks);

        for c in 0..CHANNELS {
            assert!((both_send[c] - (first_send[c] + second_send[c])).abs() < 1e-5);
        }
    }
}
