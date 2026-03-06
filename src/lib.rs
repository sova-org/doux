#[cfg(feature = "native")]
pub mod audio;
#[cfg(feature = "native")]
pub mod config;
pub mod dsp;
pub mod effects;
#[cfg(feature = "native")]
pub mod error;
pub mod event;
pub mod orbit;
#[cfg(feature = "native")]
pub mod osc;
#[cfg(feature = "native")]
mod recorder;
pub mod plaits;
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

use dsp::{fast_tanh_f32, init_envelope};
use event::Event;
use effects::Lag;
use orbit::Orbit;
#[cfg(feature = "native")]
use sampling::RegistrySample;
use sampling::SampleEntry;
#[cfg(feature = "native")]
pub use sampling::SampleLoader;
#[cfg(feature = "native")]
pub use sampling::{SampleData, SampleRegistry};
#[cfg(feature = "native")]
use recorder::Recorder;
#[cfg(not(feature = "native"))]
use sampling::{SampleInfo, SamplePool};
use schedule::Schedule;
#[cfg(feature = "native")]
use std::sync::Arc;
#[cfg(feature = "native")]
pub use telemetry::EngineMetrics;
use types::{Source, BLOCK_SIZE, CHANNELS, DEFAULT_MAX_VOICES, MAX_ORBITS};
use voice::{Voice, VoiceParams};

#[cfg(feature = "soundfont")]
struct GmResolved {
    data: Arc<SampleData>,
    name: String,
    root_freq: f32,
    loop_start: f32,
    loop_end: f32,
    looping: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
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
    voice_gain_lag: [Lag; MAX_ORBITS],
}

impl Engine {
    #[cfg(not(feature = "native"))]
    pub fn new(sample_rate: f32) -> Self {
        Self::new_with_channels(sample_rate, CHANNELS, DEFAULT_MAX_VOICES)
    }

    #[cfg(not(feature = "native"))]
    pub fn new_with_channels(sample_rate: f32, output_channels: usize, max_voices: usize) -> Self {
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
            output: vec![0.0; BLOCK_SIZE * output_channels],
            sample_pool: SamplePool::new(),
            samples: Vec::with_capacity(256),
            sample_index: Vec::new(),
            voice_gain_lag: [Lag { s: 1.0 }; MAX_ORBITS],

        }
    }

    #[cfg(feature = "native")]
    pub fn new(sample_rate: f32) -> Self {
        Self::new_with_channels(sample_rate, CHANNELS, DEFAULT_MAX_VOICES)
    }

    #[cfg(feature = "native")]
    pub fn new_with_channels(sample_rate: f32, output_channels: usize, max_voices: usize) -> Self {
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
            output: vec![0.0; BLOCK_SIZE * output_channels],
            sample_index: Vec::new(),
            sample_registry: registry,
            sample_loader: loader,
            recorder: Recorder::new(sample_rate),
            orbit_rec_bus: vec![0.0; MAX_ORBITS * BLOCK_SIZE * CHANNELS],
            metrics: Arc::new(EngineMetrics::default()),
            #[cfg(feature = "soundfont")]
            gm_bank: None,
            voice_gain_lag: [Lag { s: 1.0 }; MAX_ORBITS],

        }
    }

    #[cfg(feature = "native")]
    pub fn new_with_metrics(
        sample_rate: f32,
        output_channels: usize,
        max_voices: usize,
        metrics: Arc<EngineMetrics>,
    ) -> Self {
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
            output: vec![0.0; BLOCK_SIZE * output_channels],
            sample_index: Vec::new(),
            sample_registry: registry,
            sample_loader: loader,
            recorder: Recorder::new(sample_rate),
            orbit_rec_bus: vec![0.0; MAX_ORBITS * BLOCK_SIZE * CHANNELS],
            metrics,
            #[cfg(feature = "soundfont")]
            gm_bank: None,
            voice_gain_lag: [Lag { s: 1.0 }; MAX_ORBITS],

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
    fn get_sample_name(&self, name: &str, n: usize) -> Option<String> {
        let index_idx = self.find_sample_index(name, n)?;
        Some(self.sample_index[index_idx].name.clone())
    }

    /// Try to get a sample from the registry, or request background loading.
    #[cfg(feature = "native")]
    fn get_registry_sample(&mut self, name: &str, n: usize) -> Option<(String, Arc<SampleData>)> {
        let sample_name = self.get_sample_name(name, n)?;

        if let Some(data) = self.sample_registry.get(&sample_name) {
            if data.frame_count < data.total_frames {
                let index_idx = self.find_sample_index(name, n)?;
                let path = self.sample_index[index_idx].path.clone();
                self.sample_loader
                    .request(sample_name.clone(), path, self.sr);
            }
            return Some((sample_name, data));
        }

        let index_idx = self.find_sample_index(name, n)?;
        let path = self.sample_index[index_idx].path.clone();
        self.sample_loader.request(sample_name, path, self.sr);

        None
    }

    /// Resolve a GM soundfont zone: extract program from sound string, look up zone, get sample.
    #[cfg(feature = "soundfont")]
    fn resolve_gm(&self, event: &Event) -> Option<GmResolved> {
        let sound_str = event.sound.as_ref()?;
        if sound_str != "gm" {
            return None;
        }
        let program_str = event.n.as_deref().unwrap_or("0");

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
            name: zone.sample_name.to_string(),
            root_freq: zone.root_freq,
            loop_start: zone.loop_start,
            loop_end: zone.loop_end,
            looping: zone.looping,
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

    pub fn evaluate(&mut self, input: &str) -> Option<usize> {
        let event = Event::parse(input, self.sr);

        // Default to "play" if no explicit command - matches dough's JS wrapper behavior
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
                        self.voices[v].params.gate = 0.0;
                    }
                }
                None
            }
            "hush_endless" => {
                for i in 0..self.active_voices {
                    if self.voices[i].params.duration.is_none() {
                        self.voices[i].params.gate = 0.0;
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
            event.tick = Some(event.tick.unwrap_or(self.tick) + delta);
            event.delta = None;
        }
        if event.tick.is_some() {
            self.schedule.push(event);
            return None;
        }
        self.process_event(&event)
    }

    #[cfg(feature = "native")]
    fn handle_rec(&mut self, event: &Event) {
        let overdub = event.overdub.unwrap_or(false);
        let name = event.sound.as_deref();
        let orbit = event.orbit;

        if self.recorder.toggle(name, overdub, orbit, &self.sample_registry).is_some() {
            if let Some((name, data)) = self.recorder.finalize() {
                let key = format!("{name}/0");
                self.sample_registry.insert(key.clone(), data);
                if !self.sample_index.iter().any(|e| e.name == key) {
                    self.sample_index.push(SampleEntry {
                        name: key,
                        path: std::path::PathBuf::new(),
                    });
                }

            }
        }
    }

    pub fn play(&mut self, params: VoiceParams) -> Option<usize> {
        if self.active_voices >= self.max_voices {
            return None;
        }
        let i = self.active_voices;
        self.voices[i] = Voice::default();
        self.voices[i].params = params;
        self.voices[i].sr = self.sr;
        self.active_voices += 1;
        Some(i)
    }

    /// Process an event, handling voice selection like dough.c's process_engine_event()
    fn process_event(&mut self, event: &Event) -> Option<usize> {
        // Cut group: release any voices in the same cut group
        if let Some(cut) = event.cut {
            for i in 0..self.active_voices {
                if self.voices[i].params.cut == Some(cut) {
                    self.voices[i].params.gate = 0.0;
                }
            }
        }

        // If sound is specified but doesn't resolve to anything, check availability
        // Skip this check if WebSample data is already present (WASM with JS-loaded sample)
        let has_web_sample = event.file_pcm.is_some() && event.file_frames.is_some();
        if let Some(ref sound_str) = event.sound {
            if !has_web_sample && sound_str.parse::<Source>().is_err() {
                let effective_name = match &event.bank {
                    Some(b) => format!("{sound_str}_{b}"),
                    None => sound_str.clone(),
                };
                // Check if sample is loaded. If not, request loading and skip this event.
                #[cfg(feature = "native")]
                {
                    let n = event.n_as_index();
                    self.get_registry_sample(&effective_name, n)?;
                }
                #[cfg(not(feature = "native"))]
                {
                    let n = event.n_as_index();
                    self.get_or_load_sample(&effective_name, n)?;
                }
            }
        }

        let (voice_idx, is_new_voice) = if let Some(v) = event.voice {
            if v < self.active_voices {
                // Voice exists - reuse it
                (v, false)
            } else {
                // Voice index out of range - allocate new
                if self.active_voices >= self.max_voices {
                    return None;
                }
                let i = self.active_voices;
                self.active_voices += 1;
                (i, true)
            }
        } else {
            // No voice specified - allocate new
            if self.active_voices >= self.max_voices {
                return None;
            }
            let i = self.active_voices;
            self.active_voices += 1;
            (i, true)
        };

        let should_reset = is_new_voice || event.reset.unwrap_or(false);

        if should_reset {
            self.voices[voice_idx] = Voice::default();
            self.voices[voice_idx].sr = self.sr;
            // Initialize glide_lag to target freq to prevent glide from 0
            if let Some(freq) = event.freq {
                self.voices[voice_idx].glide_lag.s = freq;
            }
        }

        // Update voice params (only the ones explicitly set in event)
        self.update_voice_params(voice_idx, event);

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
                    let effective_name = match &event.bank {
                        Some(b) => format!("{sound_str}_{b}"),
                        None => sound_str.clone(),
                    };
                    let n_float = event.n_as_float();
                    let n_floor = n_float.floor() as usize;
                    let blend = n_float.fract();
                    let a = self.get_registry_sample(&effective_name, n_floor);
                    let b = if blend > 0.0 {
                        self.get_registry_sample(&effective_name, n_floor + 1)
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
                let effective_name = match &event.bank {
                    Some(b) => format!("{sound_str}_{b}"),
                    None => sound_str.clone(),
                };
                let n = event.n_as_index();
                self.get_or_load_sample(&effective_name, n)
            } else {
                None
            }
        } else {
            None
        };

        let v = &mut self.voices[idx];

        // --- Pitch ---
        copy_opt!(event, v.params, freq, detune, speed);
        copy_opt_some!(event, v.params, glide);

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
        if let Some(mult) = event.mult {
            v.params.shape.mult = mult.clamp(0.25, 16.0);
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
            let mut rs = RegistrySample::new(gm.name, gm.data, 0.0, 1.0);
            rs.root_freq = gm.root_freq;
            if gm.looping {
                rs.set_loop(gm.loop_start, gm.loop_end);
            }
            v.registry_sample = Some(rs);
            if event.freq.is_none() {
                v.params.freq = 261.626;
            }
            if event.attack.is_none() {
                v.params.attack = gm.attack;
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
        }

        // Sample playback via lock-free registry (native)
        #[cfg(feature = "native")]
        if let Some((sample_name, sample_data)) = registry_sample_data {
            // Use Wavetable mode if scan param present, otherwise Sample
            v.params.sound = if event.scan.is_some() {
                Source::Wavetable
            } else {
                Source::Sample
            };
            let (begin, end) = event.resolve_range();
            let frame_count = sample_data.total_frames;
            v.registry_sample = Some(RegistrySample::new(sample_name, sample_data, begin, end));
            if let Some((name_b, data_b)) = registry_sample_data_b {
                v.registry_sample_b = Some(RegistrySample::new(name_b, data_b, begin, end));
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
                // Use Wavetable mode if scan param present, otherwise Sample
                v.params.sound = if event.scan.is_some() {
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
            // Use Wavetable mode if scan param present, otherwise WebSample
            v.params.sound = if event.scan.is_some() {
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
        copy_opt_some!(event, v.params, duration);

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
        let gain_env = init_envelope(None, att, dec, sus, rel);
        if gain_env.active {
            v.params.attack = gain_env.att;
            v.params.decay = gain_env.dec;
            v.params.sustain = gain_env.sus;
            v.params.release = gain_env.rel;
        }

        // --- Filters ---
        // Macro to apply envelope params (env amount + ADSR) to a target
        macro_rules! apply_env {
            ($src:expr, $dst:expr, $e:ident, $a:ident, $d:ident, $s:ident, $r:ident, $active:ident) => {
                let env = init_envelope($src.$e, $src.$a, $src.$d, $src.$s, $src.$r);
                if env.active {
                    $dst.$e = env.env;
                    $dst.$a = env.att;
                    $dst.$d = env.dec;
                    $dst.$s = env.sus;
                    $dst.$r = env.rel;
                    $dst.$active = true;
                }
            };
        }

        copy_opt_some!(event, v.params, lpf);
        copy_opt!(event, v.params, lpq);
        apply_env!(event, v.params, lpe, lpa, lpd, lps, lpr, lp_env_active);

        copy_opt_some!(event, v.params, hpf);
        copy_opt!(event, v.params, hpq);
        apply_env!(event, v.params, hpe, hpa, hpd, hps, hpr, hp_env_active);

        copy_opt_some!(event, v.params, bpf);
        copy_opt!(event, v.params, bpq);
        apply_env!(event, v.params, bpe, bpa, bpd, bps, bpr, bp_env_active);

        copy_opt_some!(event, v.params, llpf);
        copy_opt!(event, v.params, llpq);
        copy_opt_some!(event, v.params, lhpf);
        copy_opt!(event, v.params, lhpq);
        copy_opt_some!(event, v.params, lbpf);
        copy_opt!(event, v.params, lbpq);

        // --- Modulation ---
        apply_env!(
            event,
            v.params,
            penv,
            patt,
            pdec,
            psus,
            prel,
            pitch_env_active
        );
        copy_opt!(event, v.params, vib, vibmod, vibshape);
        copy_opt!(event, v.params, fm, fmh, fmshape, fm2, fm2h, fmalgo, fmfb);
        apply_env!(event, v.params, fme, fma, fmd, fms, fmr, fm_env_active);
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
        copy_opt!(event, v.params, feedback, fbtime, fbdamp, fblfo, fblfodepth, fblfoshape);
        copy_opt!(event, v.params, comp, compattack, comprelease, comporbit);
        copy_opt_some!(event, v.params, coarse, crush, fold, wrap, distort);
        copy_opt!(event, v.params, distortvol);
        copy_opt!(event, v.params, width, haas);
        copy_opt!(event, v.params, eqlo, eqmid, eqhi, tilt);

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

        // Install inline parameter modulations
        for (id, chain) in &event.mods {
            v.set_mod(*id, *chain);
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
            let mut event = self.schedule.pop_front().unwrap();

            if diff < tolerance {
                self.process_event(&event);
            }

            if let Some(rep) = event.repeat {
                event.tick = Some(t + rep);
                self.schedule.push(event);
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

        let mut voices_per_orbit = [0u32; MAX_ORBITS];
        for i in 0..self.active_voices {
            voices_per_orbit[self.voices[i].params.orbit % num_orbits] += 1;
        }
        let mut voice_comp = [0.0f32; MAX_ORBITS];
        for o in 0..num_orbits {
            let target = if voices_per_orbit[o] > 0 {
                1.0 / (voices_per_orbit[o] as f32).sqrt()
            } else {
                1.0
            };
            voice_comp[o] = self.voice_gain_lag[o].update(target, 0.005, self.sr);
        }

        let mut orbit_dry = [[0.0f32; CHANNELS]; MAX_ORBITS];
        let mut i = 0;
        while i < self.active_voices {
            #[cfg(feature = "native")]
            {
                if let Some(ref mut rs) = self.voices[i].registry_sample {
                    if rs.is_head() {
                        if let Some(full) = self.sample_registry.get(&rs.name) {
                            if full.frame_count >= full.total_frames {
                                rs.upgrade(full);
                            }
                        }
                    }
                }
                if let Some(ref mut rs) = self.voices[i].registry_sample_b {
                    if rs.is_head() {
                        if let Some(full) = self.sample_registry.get(&rs.name) {
                            if full.frame_count >= full.total_frames {
                                rs.upgrade(full);
                            }
                        }
                    }
                }
            }
            #[cfg(feature = "native")]
            let alive = self.voices[i].process(isr, web_pcm, sample_idx, live_input);
            #[cfg(not(feature = "native"))]
            let alive = {
                let pool = self.sample_pool.data.as_slice();
                let samples = self.samples.as_slice();
                self.voices[i].process(isr, pool, samples, web_pcm, sample_idx, live_input)
            };
            if !alive {
                self.free_voice(i);
                continue;
            }

            let orbit_idx = self.voices[i].params.orbit % num_orbits;

            self.voices[i].ch[0] *= voice_comp[orbit_idx];
            self.voices[i].ch[1] *= voice_comp[orbit_idx];

            orbit_dry[orbit_idx][0] += self.voices[i].ch[0];
            orbit_dry[orbit_idx][1] += self.voices[i].ch[1];

            // Add to orbit sends
            if self.voices[i].params.delay > 0.0 {
                for c in 0..CHANNELS {
                    self.orbits[orbit_idx]
                        .add_delay_send(c, self.voices[i].ch[c] * self.voices[i].params.delay);
                }
                self.orbits[orbit_idx].params.delay_time = self.voices[i].params.delaytime;
                self.orbits[orbit_idx].params.delay_feedback = self.voices[i].params.delayfeedback;
                self.orbits[orbit_idx].params.delay_type = self.voices[i].params.delaytype;
            }
            if self.voices[i].params.verb > 0.0 {
                for c in 0..CHANNELS {
                    self.orbits[orbit_idx]
                        .add_verb_send(c, self.voices[i].ch[c] * self.voices[i].params.verb);
                }
                self.orbits[orbit_idx].params.verb_type = self.voices[i].params.verbtype;
                self.orbits[orbit_idx].params.verb_decay = self.voices[i].params.verbdecay;
                self.orbits[orbit_idx].params.verb_damp = self.voices[i].params.verbdamp;
                self.orbits[orbit_idx].params.verb_predelay = self.voices[i].params.verbpredelay;
                self.orbits[orbit_idx].params.verb_diff = self.voices[i].params.verbdiff;
                self.orbits[orbit_idx].params.verb_prelow = self.voices[i].params.verbprelow;
                self.orbits[orbit_idx].params.verb_prehigh = self.voices[i].params.verbprehigh;
                self.orbits[orbit_idx].params.verb_lowcut = self.voices[i].params.verblowcut;
                self.orbits[orbit_idx].params.verb_highcut = self.voices[i].params.verbhighcut;
                self.orbits[orbit_idx].params.verb_lowgain = self.voices[i].params.verblowgain;
                self.orbits[orbit_idx].params.verb_chorus = self.voices[i].params.verbchorus;
                self.orbits[orbit_idx].params.verb_chorus_freq = self.voices[i].params.verbchorusfreq;
            }
            if self.voices[i].params.comb > 0.0 {
                for c in 0..CHANNELS {
                    self.orbits[orbit_idx]
                        .add_comb_send(c, self.voices[i].ch[c] * self.voices[i].params.comb);
                }
                self.orbits[orbit_idx].params.comb_freq = self.voices[i].params.combfreq;
                self.orbits[orbit_idx].params.comb_feedback = self.voices[i].params.combfeedback;
                self.orbits[orbit_idx].params.comb_damp = self.voices[i].params.combdamp;
            }
            if self.voices[i].params.feedback > 0.0 {
                for c in 0..CHANNELS {
                    self.orbits[orbit_idx]
                        .add_fb_send(c, self.voices[i].ch[c] * self.voices[i].params.feedback);
                }
                self.orbits[orbit_idx].fb_level = self.voices[i].params.feedback;
                self.orbits[orbit_idx].params.fb_time = self.voices[i].params.fbtime;
                self.orbits[orbit_idx].params.fb_damp = self.voices[i].params.fbdamp;
                self.orbits[orbit_idx].params.fb_lfo = self.voices[i].params.fblfo;
                self.orbits[orbit_idx].params.fb_lfo_depth = self.voices[i].params.fblfodepth;
                self.orbits[orbit_idx].params.fb_lfo_shape = self.voices[i].params.fblfoshape;
            }
            if self.voices[i].params.comp > 0.0 {
                self.orbits[orbit_idx].params.comp = self.voices[i].params.comp;
                self.orbits[orbit_idx].params.comp_attack = self.voices[i].params.compattack;
                self.orbits[orbit_idx].params.comp_release = self.voices[i].params.comprelease;
                self.orbits[orbit_idx].params.comp_orbit = self.voices[i].params.comporbit;
            }

            i += 1;
        }

        // Phase 1: process all orbits, store output levels
        let mut orbit_levels = [[0.0f32; CHANNELS]; MAX_ORBITS];
        for (oi, orbit) in self.orbits.iter_mut().enumerate() {
            orbit.process();
            orbit_levels[oi] = [
                orbit.delay_out[0] + orbit.verb_out[0] + orbit.comb_out[0] + orbit.fb_out[0],
                orbit.delay_out[1] + orbit.verb_out[1] + orbit.comb_out[1] + orbit.fb_out[1],
            ];
        }

        // Phase 2: mix to output with optional sidechain compression
        let isr = self.isr;
        let num_orbs = self.orbits.len();
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

        for c in 0..self.output_channels {
            output[base_idx + c] = fast_tanh_f32(output[base_idx + c] * 0.7);
        }
    }

    pub fn process_block(&mut self, output: &mut [f32], web_pcm: &[f32], live_input: &[f32]) {
        #[cfg(feature = "native")]
        let start = std::time::Instant::now();

        let samples = output.len() / self.output_channels;

        #[cfg(feature = "native")]
        {
            let needed = MAX_ORBITS * samples * CHANNELS;
            if self.orbit_rec_bus.len() < needed {
                self.orbit_rec_bus.resize(needed, 0.0);
            }
        }

        for i in 0..samples {
            self.process_schedule();
            self.tick += 1;
            self.time = self.tick as f64 / self.sr as f64;
            self.gen_sample(output, i, samples, web_pcm, live_input);
        }

        #[cfg(feature = "native")]
        {
            let n = samples * CHANNELS;
            if let Some(oi) = self.recorder.target_orbit() {
                let start_idx = oi * samples * CHANNELS;
                self.recorder.capture_block(&self.orbit_rec_bus[start_idx..start_idx + n], samples, CHANNELS);
            } else {
                self.recorder.capture_block(output, samples, self.output_channels);
            }
        }

        #[cfg(feature = "native")]
        {
            use std::sync::atomic::Ordering;
            let elapsed_ns = start.elapsed().as_nanos() as u64;
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
        }

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
            self.voices[i].params.gate = 0.0;
        }
    }

    pub fn panic(&mut self) {
        self.active_voices = 0;
    }
}
