use std::str::FromStr;

pub const WASM_BLOCK_SIZE: usize = 128;
pub const DEFAULT_NATIVE_BLOCK_SIZE: usize = 512;
pub const CHANNELS: usize = 2;
pub const DEFAULT_MAX_VOICES: usize = 32;
pub const MAX_EVENTS: usize = 256;
pub const MAX_ORBITS: usize = 8;

// --- Metadata ---

pub struct ParamInfo {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub default: &'static str,
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ModuleGroup {
    Source,
    Synthesis,
    Effect,
}

pub struct ModuleInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub group: ModuleGroup,
    pub params: &'static [ParamInfo],
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SourceCategory {
    Oscillator,
    Noise,
    Drum,
    Sample,
    Input,
}

#[derive(Clone, Copy, Debug)]
pub struct DrumDefaults {
    pub freq: f32,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

pub struct SourceInfo {
    pub module: ModuleInfo,
    pub aliases: &'static [&'static str],
    pub category: SourceCategory,
    pub drum_defaults: Option<DrumDefaults>,
    pub debug_id: i32,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Source {
    #[default]
    Tri,
    Sine,
    Saw,
    Zaw,
    Pulse,
    Pulze,
    Add,
    Osc,
    White,
    Pink,
    Brown,
    Kick,
    Snare,
    Hat,
    Tom,
    Rim,
    Cowbell,
    Cymbal,
    Gm,
    Sample,
    Wavetable,
    WebSample,
    LiveInput,
}

const ALL_SOURCES: [Source; 23] = [
    Source::Tri, Source::Sine, Source::Saw, Source::Zaw,
    Source::Pulse, Source::Pulze, Source::Add, Source::Osc,
    Source::White, Source::Pink, Source::Brown,
    Source::Kick, Source::Snare, Source::Hat, Source::Tom,
    Source::Rim, Source::Cowbell, Source::Cymbal,
    Source::Gm, Source::Sample, Source::Wavetable,
    Source::WebSample, Source::LiveInput,
];

// --- SourceInfo static data ---

macro_rules! source_info {
    ($name:expr, $aliases:expr, $cat:expr, $desc:expr, $drums:expr, $params:expr, $id:expr) => {
        SourceInfo {
            module: ModuleInfo { name: $name, description: $desc, group: ModuleGroup::Source, params: $params },
            aliases: $aliases, category: $cat, drum_defaults: $drums, debug_id: $id,
        }
    };
}

const INFO_TRI: SourceInfo = source_info!("tri", &["triangle"], SourceCategory::Oscillator, "Triangle wave with only odd harmonics and gentle rolloff", None, &[], 0);
const INFO_SINE: SourceInfo = source_info!("sine", &[], SourceCategory::Oscillator, "Pure sine wave with no harmonics", None, &[], 1);
const INFO_SAW: SourceInfo = source_info!("saw", &["sawtooth"], SourceCategory::Oscillator, "Band-limited sawtooth wave, rich in harmonics", None, &[], 2);
const INFO_ZAW: SourceInfo = source_info!("zaw", &["zawtooth"], SourceCategory::Oscillator, "Naive sawtooth with no anti-aliasing", None, &[], 3);

const INFO_PULSE: SourceInfo = source_info!("pulse", &["square"], SourceCategory::Oscillator, "Band-limited pulse wave with controllable width", None, &[
    ParamInfo { name: "pw", aliases: &[], description: "pulse width", default: "0.5", min: 0.0, max: 1.0 },
], 4);

const INFO_PULZE: SourceInfo = source_info!("pulze", &["zquare"], SourceCategory::Oscillator, "Naive pulse with no anti-aliasing", None, &[
    ParamInfo { name: "pw", aliases: &[], description: "pulse width", default: "0.5", min: 0.0, max: 1.0 },
], 5);

const INFO_ADD: SourceInfo = source_info!("add", &[], SourceCategory::Oscillator, "Additive oscillator building timbres by stacking sine partials", None, &[
    ParamInfo { name: "timbre", aliases: &[], description: "spectral tilt", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "morph", aliases: &[], description: "even/odd partial balance", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "harmonics", aliases: &["harm"], description: "inharmonic stretch", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "partials", aliases: &[], description: "number of partials (1-32)", default: "32.0", min: 1.0, max: 32.0 },
], 6);

const INFO_OSC: SourceInfo = source_info!("osc", &["oscillator"], SourceCategory::Oscillator,
    "Morphing oscillator: sine → triangle → saw → square via wave parameter", None, &[
    ParamInfo { name: "wave", aliases: &["waveform"], description: "waveform morph (0 sine, 0.33 tri, 0.67 saw, 1 square)", default: "0.0", min: 0.0, max: 1.0 },
], 14);

const INFO_WHITE: SourceInfo = source_info!("white", &[], SourceCategory::Noise, "White noise with equal energy at all frequencies", None, &[], 7);
const INFO_PINK: SourceInfo = source_info!("pink", &[], SourceCategory::Noise, "Pink noise (1/f) with equal energy per octave", None, &[], 8);
const INFO_BROWN: SourceInfo = source_info!("brown", &[], SourceCategory::Noise, "Brown noise (1/f^2) weighted toward low frequencies", None, &[], 9);

const INFO_KICK: SourceInfo = source_info!("kick", &[], SourceCategory::Drum, "Pitched body with sweep envelope and optional saturation",
    Some(DrumDefaults { freq: 55.0, attack: 0.001, decay: 0.3, sustain: 0.0, release: 0.005 }), &[
    ParamInfo { name: "morph", aliases: &[], description: "sweep depth", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "harmonics", aliases: &["harm"], description: "sweep speed", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "timbre", aliases: &[], description: "saturation", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "wave", aliases: &["waveform"], description: "oscillator waveform (0 sine, 0.5 tri, 1 saw)", default: "0.0", min: 0.0, max: 1.0 },
], 24);

const INFO_SNARE: SourceInfo = source_info!("snare", &["sd"], SourceCategory::Drum, "Body + noise mix",
    Some(DrumDefaults { freq: 180.0, attack: 0.001, decay: 0.15, sustain: 0.0, release: 0.005 }), &[
    ParamInfo { name: "timbre", aliases: &[], description: "body/noise mix", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "harmonics", aliases: &["harm"], description: "noise brightness", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "wave", aliases: &["waveform"], description: "oscillator waveform (0 sine, 0.5 tri, 1 saw)", default: "0.0", min: 0.0, max: 1.0 },
], 25);

const INFO_HAT: SourceInfo = source_info!("hat", &["hh", "hihat"], SourceCategory::Drum, "Phase-modulated metallic tone through a resonant lowpass",
    Some(DrumDefaults { freq: 320.0, attack: 0.001, decay: 0.08, sustain: 0.0, release: 0.005 }), &[
    ParamInfo { name: "morph", aliases: &[], description: "clean to metallic", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "harmonics", aliases: &["harm"], description: "dark to bright", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "timbre", aliases: &[], description: "filter resonance", default: "0.5", min: 0.0, max: 1.0 },
], 26);

const INFO_TOM: SourceInfo = source_info!("tom", &[], SourceCategory::Drum, "Pitched body with gentle sweep and optional noise",
    Some(DrumDefaults { freq: 120.0, attack: 0.001, decay: 0.25, sustain: 0.0, release: 0.005 }), &[
    ParamInfo { name: "morph", aliases: &[], description: "sweep depth", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "harmonics", aliases: &["harm"], description: "sweep speed", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "timbre", aliases: &[], description: "noise amount", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "wave", aliases: &["waveform"], description: "oscillator waveform (0 sine, 0.5 tri, 1 saw)", default: "0.0", min: 0.0, max: 1.0 },
], 27);

const INFO_RIM: SourceInfo = source_info!("rim", &["rimshot", "rs"], SourceCategory::Drum, "Short pitched click with noise",
    Some(DrumDefaults { freq: 400.0, attack: 0.001, decay: 0.04, sustain: 0.0, release: 0.005 }), &[
    ParamInfo { name: "morph", aliases: &[], description: "pitch sweep", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "harmonics", aliases: &["harm"], description: "noise brightness", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "timbre", aliases: &[], description: "body/noise mix", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "wave", aliases: &["waveform"], description: "oscillator waveform (0 sine, 0.5 tri, 1 saw)", default: "0.0", min: 0.0, max: 1.0 },
], 29);

const INFO_COWBELL: SourceInfo = source_info!("cowbell", &["cb"], SourceCategory::Drum, "Two detuned oscillators through a bandpass",
    Some(DrumDefaults { freq: 540.0, attack: 0.001, decay: 0.12, sustain: 0.0, release: 0.005 }), &[
    ParamInfo { name: "morph", aliases: &[], description: "detune amount", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "harmonics", aliases: &["harm"], description: "brightness", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "timbre", aliases: &[], description: "metallic bite", default: "0.5", min: 0.0, max: 1.0 },
], 30);

const INFO_CYMBAL: SourceInfo = source_info!("cymbal", &["cy"], SourceCategory::Drum, "Inharmonic metallic wash with filtered noise",
    Some(DrumDefaults { freq: 420.0, attack: 0.001, decay: 0.5, sustain: 0.0, release: 0.005 }), &[
    ParamInfo { name: "morph", aliases: &[], description: "ratio spread (bell-like to crash)", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "harmonics", aliases: &["harm"], description: "brightness (dark to sizzly)", default: "0.5", min: 0.0, max: 1.0 },
    ParamInfo { name: "timbre", aliases: &[], description: "noise amount", default: "0.5", min: 0.0, max: 1.0 },
], 31);

const INFO_GM: SourceInfo = source_info!("gm", &[], SourceCategory::Sample, "General MIDI via soundfont", None, &[], 32);

const INFO_SAMPLE: SourceInfo = source_info!("sample", &[], SourceCategory::Sample, "Disk-loaded audio sample playback", None, &[
    ParamInfo { name: "n", aliases: &[], description: "sample index within folder", default: "0.0", min: 0.0, max: f32::MAX },
    ParamInfo { name: "begin", aliases: &[], description: "start position (0-1)", default: "0.0", min: 0.0, max: 1.0 },
    ParamInfo { name: "end", aliases: &[], description: "end position (0-1)", default: "1.0", min: 0.0, max: 1.0 },
    ParamInfo { name: "speed", aliases: &[], description: "playback speed", default: "1.0", min: -100.0, max: 100.0 },
    ParamInfo { name: "stretch", aliases: &[], description: "time stretch factor", default: "1.0", min: 0.0, max: 100.0 },
    ParamInfo { name: "cut", aliases: &[], description: "choke group", default: "0.0", min: 0.0, max: f32::MAX },
], 10);

const INFO_WAVETABLE: SourceInfo = source_info!("wt", &[], SourceCategory::Sample, "Sample played as wavetable oscillator with pitch tracking", None, &[
    ParamInfo { name: "scan", aliases: &[], description: "wavetable position (0-1)", default: "0.0", min: 0.0, max: 1.0 },
    ParamInfo { name: "wtlen", aliases: &[], description: "cycle length in samples", default: "0.0", min: 0.0, max: 2048.0 },
], 11);

const INFO_WEBSAMPLE: SourceInfo = source_info!("websample", &[], SourceCategory::Sample, "Inline PCM sample from JavaScript", None, &[
    ParamInfo { name: "begin", aliases: &[], description: "start position (0-1)", default: "0.0", min: 0.0, max: 1.0 },
    ParamInfo { name: "end", aliases: &[], description: "end position (0-1)", default: "1.0", min: 0.0, max: 1.0 },
    ParamInfo { name: "speed", aliases: &[], description: "playback speed", default: "1.0", min: -100.0, max: 100.0 },
], 12);

const INFO_LIVEINPUT: SourceInfo = source_info!("live", &["mic"], SourceCategory::Input, "Live audio input (microphone, line-in)", None, &[], 13);

impl Source {
    pub const fn all() -> &'static [Source] {
        &ALL_SOURCES
    }

    pub const fn info(&self) -> &'static SourceInfo {
        match self {
            Self::Tri => &INFO_TRI,
            Self::Sine => &INFO_SINE,
            Self::Saw => &INFO_SAW,
            Self::Zaw => &INFO_ZAW,
            Self::Pulse => &INFO_PULSE,
            Self::Pulze => &INFO_PULZE,
            Self::Add => &INFO_ADD,
            Self::Osc => &INFO_OSC,
            Self::White => &INFO_WHITE,
            Self::Pink => &INFO_PINK,
            Self::Brown => &INFO_BROWN,
            Self::Kick => &INFO_KICK,
            Self::Snare => &INFO_SNARE,
            Self::Hat => &INFO_HAT,
            Self::Tom => &INFO_TOM,
            Self::Rim => &INFO_RIM,
            Self::Cowbell => &INFO_COWBELL,
            Self::Cymbal => &INFO_CYMBAL,
            Self::Gm => &INFO_GM,
            Self::Sample => &INFO_SAMPLE,
            Self::Wavetable => &INFO_WAVETABLE,
            Self::WebSample => &INFO_WEBSAMPLE,
            Self::LiveInput => &INFO_LIVEINPUT,
        }
    }

    pub fn drum_defaults(&self) -> Option<(f32, f32, f32, f32, f32)> {
        self.info().drum_defaults.map(|d| (d.freq, d.attack, d.decay, d.sustain, d.release))
    }
}

impl FromStr for Source {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for &source in Source::all() {
            let info = source.info();
            if info.module.name == s {
                return Ok(source);
            }
            for &alias in info.aliases {
                if alias == s {
                    return Ok(source);
                }
            }
        }
        Err(())
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum SubWave {
    #[default]
    Tri,
    Sine,
    Square,
}

impl FromStr for SubWave {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tri" => Ok(Self::Tri),
            "sine" => Ok(Self::Sine),
            "square" | "pulse" => Ok(Self::Square),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum LfoShape {
    #[default]
    Sine,
    Tri,
    Saw,
    Square,
    Sh,
}

impl FromStr for LfoShape {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sine" | "sin" => Ok(Self::Sine),
            "tri" | "triangle" => Ok(Self::Tri),
            "saw" | "sawtooth" => Ok(Self::Saw),
            "square" | "pulse" => Ok(Self::Square),
            "sh" | "sah" | "random" => Ok(Self::Sh),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ReverbType {
    Plate,
    #[default]
    Space,
}

impl FromStr for ReverbType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plate" | "dattorro" | "0" => Ok(Self::Plate),
            "space" | "vital" | "1" => Ok(Self::Space),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum DelayType {
    #[default]
    Standard,
    PingPong,
    Tape,
    Multitap,
}

impl FromStr for DelayType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standard" | "std" | "0" => Ok(Self::Standard),
            "pingpong" | "pp" | "1" => Ok(Self::PingPong),
            "tape" | "2" => Ok(Self::Tape),
            "multitap" | "multi" | "3" => Ok(Self::Multitap),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
    Allpass,
    Peaking,
    Lowshelf,
    Highshelf,
}

pub fn midi2freq(note: f32) -> f32 {
    2.0_f32.powf((note - 69.0) / 12.0) * 440.0
}

pub fn freq2midi(freq: f32) -> f32 {
    let safe_freq = freq.max(0.001);
    69.0 + 12.0 * (safe_freq / 440.0).log2()
}
