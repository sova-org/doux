//! Core engine constants, the `Source` catalog, and the typed param vocabulary.

use std::str::FromStr;

/// Default cpal device buffer size on wasm builds (samples per audio quantum).
pub const WASM_BUFFER_SIZE: usize = 128;
/// Default cpal device buffer size on native builds (samples per callback).
pub const DEFAULT_BUFFER_SIZE: usize = 512;
pub const CHANNELS: usize = 2;
pub const DEFAULT_MAX_VOICES: usize = 64;
pub const MAX_EVENTS: usize = 256;
pub const MAX_ORBITS: usize = 8;
/// Hard ceiling on the frames `process_block` may receive in one callback.
/// cpal can hand back periods larger than the requested `host_buffer_size`
/// (aggregate CoreAudio devices, JACK, PipeWire), so engine-owned per-block
/// buffers are sized to this, not to the configured size.
pub const MAX_BUFFER_FRAMES: usize = 8192;

/// Bound on the control→audio command channel. Larger than the per-block
/// drain budget (64) so a few backed-up events survive one slow callback;
/// finite so a runaway sender cannot drag the system into unbounded growth.
/// Drops past this cap are counted in `EngineMetrics::dropped_cmds`.
pub const AUDIO_CMD_QUEUE_DEPTH: usize = 256;

/// Hard ceiling for the DSP inner-block size (`Engine::dsp_block_size`).
/// Sizes per-voice and per-orbit scratch buffers.
pub const MAX_BLOCK: usize = 256;

/// Default inner DSP block size in samples.
pub const DEFAULT_DSP_BLOCK_SIZE: usize = 32;

/// Worst-case sample rate the engine is sized to support. All sample-rate-
/// dependent buffer constants (delay lines in `effects/`) are derived from
/// this so their *time* meaning is preserved across host sample rates.
/// Raise to 192_000 if 192 kHz support is needed — memory cost scales linearly
/// (Feedback alone grows from ~512 KB to ~1 MB per orbit).
pub const MAX_SAMPLE_RATE: usize = 96_000;

/// In-range DSP block size, clamped to `[1, MAX_BLOCK]` at construction.
///
/// Sizes scratch buffers that are pre-allocated to `MAX_BLOCK`; the type
/// guarantees the `.get()` value never exceeds the buffer length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DspBlockSize(usize);

impl DspBlockSize {
    pub const MIN: usize = 1;
    pub const MAX: usize = MAX_BLOCK;

    pub fn new(n: usize) -> Self {
        Self(n.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> usize {
        self.0
    }
}

impl Default for DspBlockSize {
    fn default() -> Self {
        Self(DEFAULT_DSP_BLOCK_SIZE)
    }
}

/// One interleaved stereo sample.
pub type StereoFrame = [f32; CHANNELS];

// --- Metadata ---

pub struct ParamInfo {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub default: &'static str,
    pub min: f32,
    pub max: f32,
}

/// The three generic per-source parameter slots (Mutable-style macro lineage).
/// Each source gives them semantic names in its `ParamInfo` table; a param
/// resolves to a slot iff the slot's generic name (`timbre`, `harmonics`/
/// `harm`, `morph`) appears in its `aliases`. The alias doubles as the
/// documented universal fallback name, so it is load-bearing, not legacy.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GenericSlot {
    Timbre,
    Harmonics,
    Morph,
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

impl ModuleInfo {
    /// Resolve a per-source semantic key ("bright" on pluck, "drive" on kick)
    /// to its generic slot. Linear scan over the const param table; runs at
    /// event-parse time only, never in the audio hot path. Returns None for
    /// non-slot params (wave, scan, ...) and unknown keys.
    ///
    /// The flat match in `Event::parse` wins over this lookup, so a semantic
    /// name must never reuse a global key (`tilt`, `fold`, `sustain`, ...).
    pub fn semantic_slot(&self, key: &str) -> Option<GenericSlot> {
        let param = self
            .params
            .iter()
            .find(|p| p.name == key || p.aliases.contains(&key))?;
        for &alias in param.aliases {
            match alias {
                "timbre" => return Some(GenericSlot::Timbre),
                "harmonics" | "harm" => return Some(GenericSlot::Harmonics),
                "morph" => return Some(GenericSlot::Morph),
                _ => {}
            }
        }
        None
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SourceCategory {
    Oscillator,
    Noise,
    Drum,
    Sample,
    Input,
    Patch,
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
    Osc,
    Pluck,
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
    /// User-defined arf graph, resolved by its bare `<name>` as a sound.
    /// The patch handle lives on the voice (`Voice::patch`), not here.
    Arf,
}

const ALL_SOURCES: [Source; 24] = [
    Source::Tri,
    Source::Sine,
    Source::Saw,
    Source::Zaw,
    Source::Pulse,
    Source::Pulze,
    Source::Osc,
    Source::Pluck,
    Source::White,
    Source::Pink,
    Source::Brown,
    Source::Kick,
    Source::Snare,
    Source::Hat,
    Source::Tom,
    Source::Rim,
    Source::Cowbell,
    Source::Cymbal,
    Source::Gm,
    Source::Sample,
    Source::Wavetable,
    Source::WebSample,
    Source::LiveInput,
    Source::Arf,
];

// --- SourceInfo static data ---

macro_rules! source_info {
    ($name:expr, $aliases:expr, $cat:expr, $desc:expr, $drums:expr, $params:expr, $id:expr) => {
        SourceInfo {
            module: ModuleInfo {
                name: $name,
                description: $desc,
                group: ModuleGroup::Source,
                params: $params,
            },
            aliases: $aliases,
            category: $cat,
            drum_defaults: $drums,
            debug_id: $id,
        }
    };
}

const INFO_TRI: SourceInfo = source_info!(
    "tri",
    &["triangle"],
    SourceCategory::Oscillator,
    "Triangle wave with only odd harmonics and gentle rolloff",
    None,
    &[],
    0
);
const INFO_SINE: SourceInfo = source_info!(
    "sine",
    &[],
    SourceCategory::Oscillator,
    "Pure sine wave with no harmonics",
    None,
    &[],
    1
);
const INFO_SAW: SourceInfo = source_info!(
    "saw",
    &["sawtooth"],
    SourceCategory::Oscillator,
    "Band-limited sawtooth wave, rich in harmonics",
    None,
    &[],
    2
);
const INFO_ZAW: SourceInfo = source_info!(
    "zaw",
    &["zawtooth"],
    SourceCategory::Oscillator,
    "Naive sawtooth with no anti-aliasing",
    None,
    &[],
    3
);

const INFO_PULSE: SourceInfo = source_info!(
    "pulse",
    &["square"],
    SourceCategory::Oscillator,
    "Band-limited pulse wave with controllable width",
    None,
    &[ParamInfo {
        name: "pw",
        aliases: &[],
        description: "pulse width",
        default: "0.5",
        min: 0.0,
        max: 1.0
    },],
    4
);

const INFO_PULZE: SourceInfo = source_info!(
    "pulze",
    &["zquare"],
    SourceCategory::Oscillator,
    "Naive pulse with no anti-aliasing",
    None,
    &[ParamInfo {
        name: "pw",
        aliases: &[],
        description: "pulse width",
        default: "0.5",
        min: 0.0,
        max: 1.0
    },],
    5
);

const INFO_OSC: SourceInfo = source_info!(
    "osc",
    &["oscillator"],
    SourceCategory::Oscillator,
    "Morphing oscillator: sine → triangle → saw → square via wave parameter",
    None,
    &[ParamInfo {
        name: "wave",
        aliases: &["waveform"],
        description: "waveform morph (0 sine, 0.33 tri, 0.67 saw, 1 square)",
        default: "0.0",
        min: 0.0,
        max: 1.0
    },],
    14
);

const INFO_PLUCK: SourceInfo = source_info!(
    "pluck",
    &["ks", "string"],
    SourceCategory::Oscillator,
    "Karplus-Strong plucked string",
    None,
    &[
        ParamInfo {
            name: "bright",
            aliases: &["timbre"],
            description: "brightness (loop damping)",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "ring",
            aliases: &["harmonics", "harm"],
            description: "sustain (loop feedback)",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "excite",
            aliases: &["morph"],
            description: "excitation color (dark to snappy)",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
    ],
    34
);

const INFO_WHITE: SourceInfo = source_info!(
    "white",
    &[],
    SourceCategory::Noise,
    "White noise with equal energy at all frequencies",
    None,
    &[],
    7
);
const INFO_PINK: SourceInfo = source_info!(
    "pink",
    &[],
    SourceCategory::Noise,
    "Pink noise (1/f) with equal energy per octave",
    None,
    &[],
    8
);
const INFO_BROWN: SourceInfo = source_info!(
    "brown",
    &[],
    SourceCategory::Noise,
    "Brown noise (1/f^2) weighted toward low frequencies",
    None,
    &[],
    9
);

const INFO_KICK: SourceInfo = source_info!(
    "kick",
    &[],
    SourceCategory::Drum,
    "Pitched body with sweep envelope and optional saturation",
    Some(DrumDefaults {
        freq: 55.0,
        attack: 0.001,
        decay: 0.3,
        sustain: 0.0,
        release: 0.005
    }),
    &[
        ParamInfo {
            name: "sweep",
            aliases: &["morph"],
            description: "sweep depth",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "punch",
            aliases: &["harmonics", "harm"],
            description: "sweep speed",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "drive",
            aliases: &["timbre"],
            description: "saturation",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "wave",
            aliases: &["waveform"],
            description: "oscillator waveform (0 sine, 0.5 tri, 1 saw)",
            default: "0.0",
            min: 0.0,
            max: 1.0
        },
    ],
    24
);

const INFO_SNARE: SourceInfo = source_info!(
    "snare",
    &["sd"],
    SourceCategory::Drum,
    "Body + noise mix",
    Some(DrumDefaults {
        freq: 180.0,
        attack: 0.001,
        decay: 0.15,
        sustain: 0.0,
        release: 0.005
    }),
    &[
        ParamInfo {
            name: "snappy",
            aliases: &["timbre"],
            description: "body/noise mix",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "bright",
            aliases: &["harmonics", "harm"],
            description: "noise brightness",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "wave",
            aliases: &["waveform"],
            description: "oscillator waveform (0 sine, 0.5 tri, 1 saw)",
            default: "0.0",
            min: 0.0,
            max: 1.0
        },
    ],
    25
);

const INFO_HAT: SourceInfo = source_info!(
    "hat",
    &["hh", "hihat"],
    SourceCategory::Drum,
    "Phase-modulated metallic tone through a resonant lowpass",
    Some(DrumDefaults {
        freq: 320.0,
        attack: 0.001,
        decay: 0.08,
        sustain: 0.0,
        release: 0.005
    }),
    &[
        ParamInfo {
            name: "metal",
            aliases: &["morph"],
            description: "clean to metallic (ratio spread)",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "bright",
            aliases: &["harmonics", "harm"],
            description: "dark to bright (filter cutoff)",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "reso",
            aliases: &["timbre"],
            description: "filter resonance",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
    ],
    26
);

const INFO_TOM: SourceInfo = source_info!(
    "tom",
    &[],
    SourceCategory::Drum,
    "Pitched body with gentle sweep and optional noise",
    Some(DrumDefaults {
        freq: 120.0,
        attack: 0.001,
        decay: 0.25,
        sustain: 0.0,
        release: 0.005
    }),
    &[
        ParamInfo {
            name: "sweep",
            aliases: &["morph"],
            description: "sweep depth",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "punch",
            aliases: &["harmonics", "harm"],
            description: "sweep speed",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "noise",
            aliases: &["timbre"],
            description: "stick-noise amount",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "wave",
            aliases: &["waveform"],
            description: "oscillator waveform (0 sine, 0.5 tri, 1 saw)",
            default: "0.0",
            min: 0.0,
            max: 1.0
        },
    ],
    27
);

const INFO_RIM: SourceInfo = source_info!(
    "rim",
    &["rimshot", "rs"],
    SourceCategory::Drum,
    "Short pitched click with noise",
    Some(DrumDefaults {
        freq: 400.0,
        attack: 0.001,
        decay: 0.04,
        sustain: 0.0,
        release: 0.005
    }),
    &[
        ParamInfo {
            name: "shift",
            aliases: &["morph"],
            description: "upper partial shift",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "bright",
            aliases: &["harmonics", "harm"],
            description: "click brightness",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "ring",
            aliases: &["timbre"],
            description: "ring length",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "wave",
            aliases: &["waveform"],
            description: "oscillator waveform (0 sine, 0.5 tri, 1 saw)",
            default: "0.0",
            min: 0.0,
            max: 1.0
        },
    ],
    29
);

const INFO_COWBELL: SourceInfo = source_info!(
    "cowbell",
    &["cb"],
    SourceCategory::Drum,
    "Two detuned oscillators through a bandpass",
    Some(DrumDefaults {
        freq: 540.0,
        attack: 0.001,
        decay: 0.12,
        sustain: 0.0,
        release: 0.005
    }),
    &[
        ParamInfo {
            name: "clang",
            aliases: &["morph"],
            description: "detune amount",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "bright",
            aliases: &["harmonics", "harm"],
            description: "brightness (bandpass center)",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "drive",
            aliases: &["timbre"],
            description: "metallic bite (saturation)",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
    ],
    30
);

const INFO_CYMBAL: SourceInfo = source_info!(
    "cymbal",
    &["cy"],
    SourceCategory::Drum,
    "Inharmonic metallic wash with filtered noise",
    Some(DrumDefaults {
        freq: 420.0,
        attack: 0.001,
        decay: 0.5,
        sustain: 0.0,
        release: 0.005
    }),
    &[
        ParamInfo {
            name: "metal",
            aliases: &["morph"],
            description: "ratio spread (bell-like to crash)",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "bright",
            aliases: &["harmonics", "harm"],
            description: "brightness (dark to sizzly)",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "sizzle",
            aliases: &["timbre"],
            description: "noise tail amount",
            default: "0.5",
            min: 0.0,
            max: 1.0
        },
    ],
    31
);

const INFO_GM: SourceInfo = source_info!(
    "gm",
    &[],
    SourceCategory::Sample,
    "General MIDI via soundfont",
    None,
    &[],
    32
);

const INFO_SAMPLE: SourceInfo = source_info!(
    "sample",
    &[],
    SourceCategory::Sample,
    "Disk-loaded audio sample playback",
    None,
    &[
        ParamInfo {
            name: "n",
            aliases: &[],
            description: "sample index within folder",
            default: "0.0",
            min: 0.0,
            max: f32::MAX
        },
        ParamInfo {
            name: "begin",
            aliases: &[],
            description: "start position (0-1)",
            default: "0.0",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "end",
            aliases: &[],
            description: "end position (0-1)",
            default: "1.0",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "speed",
            aliases: &[],
            description: "playback speed",
            default: "1.0",
            min: -100.0,
            max: 100.0
        },
        ParamInfo {
            name: "stretch",
            aliases: &[],
            description: "time stretch factor",
            default: "1.0",
            min: 0.0,
            max: 100.0
        },
        ParamInfo {
            name: "cut",
            aliases: &[],
            description: "choke group",
            default: "0.0",
            min: 0.0,
            max: f32::MAX
        },
    ],
    10
);

const INFO_WAVETABLE: SourceInfo = source_info!(
    "wt",
    &[],
    SourceCategory::Sample,
    "Sample played as wavetable oscillator with pitch tracking",
    None,
    &[
        ParamInfo {
            name: "scan",
            aliases: &[],
            description: "wavetable position (0-1)",
            default: "0.0",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "wtlen",
            aliases: &[],
            description: "cycle length in samples",
            default: "0.0",
            min: 0.0,
            max: 2048.0
        },
    ],
    11
);

const INFO_WEBSAMPLE: SourceInfo = source_info!(
    "websample",
    &[],
    SourceCategory::Sample,
    "Inline PCM sample from JavaScript",
    None,
    &[
        ParamInfo {
            name: "begin",
            aliases: &[],
            description: "start position (0-1)",
            default: "0.0",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "end",
            aliases: &[],
            description: "end position (0-1)",
            default: "1.0",
            min: 0.0,
            max: 1.0
        },
        ParamInfo {
            name: "speed",
            aliases: &[],
            description: "playback speed",
            default: "1.0",
            min: -100.0,
            max: 100.0
        },
    ],
    12
);

const INFO_LIVEINPUT: SourceInfo = source_info!(
    "live",
    &["mic"],
    SourceCategory::Input,
    "Live audio input (microphone, line-in)",
    None,
    &[],
    13
);

const INFO_ARF: SourceInfo = source_info!(
    "arf",
    &[],
    SourceCategory::Patch,
    "User-defined arf graph patch, triggered by its bare name",
    None,
    &[],
    15
);

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
            Self::Osc => &INFO_OSC,
            Self::Pluck => &INFO_PLUCK,
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
            Self::Arf => &INFO_ARF,
        }
    }

    pub fn drum_defaults(&self) -> Option<(f32, f32, f32, f32, f32)> {
        self.info()
            .drum_defaults
            .map(|d| (d.freq, d.attack, d.decay, d.sustain, d.release))
    }

    /// Returns documentation metadata for all sources.
    /// Each entry: (name, aliases, description, category, param descriptions).
    pub fn all_source_docs() -> Vec<SourceDoc> {
        Source::all()
            .iter()
            .map(|s| {
                let info = s.info();
                let params: Vec<(&str, &str)> = info
                    .module
                    .params
                    .iter()
                    .map(|p| (p.name, p.description))
                    .collect();
                SourceDoc {
                    name: info.module.name,
                    aliases: info.aliases,
                    description: info.module.description,
                    category: match info.category {
                        SourceCategory::Oscillator => "Oscillator",
                        SourceCategory::Noise => "Noise",
                        SourceCategory::Drum => "Drum",
                        SourceCategory::Sample => "Sample",
                        SourceCategory::Input => "Input",
                        SourceCategory::Patch => "Patch",
                    },
                    params,
                }
            })
            .collect()
    }
}

pub struct SourceDoc {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub category: &'static str,
    pub params: Vec<(&'static str, &'static str)>,
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
        #[cfg(feature = "soundfont")]
        if s.starts_with("gm")
            && s.len() > 2
            && crate::soundfont::resolve_gm_program(&s[2..]).is_some()
        {
            return Ok(Source::Gm);
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
    Cloud,
    #[default]
    Space,
}

impl FromStr for ReverbType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cloud" | "jpverb" | "0" => Ok(Self::Cloud),
            "space" | "vital" | "1" => Ok(Self::Space),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum SyncMode {
    #[default]
    Hard,
    Soft,
}

impl FromStr for SyncMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hard" | "0" => Ok(Self::Hard),
            "soft" | "1" => Ok(Self::Soft),
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

/// Saturator curve for the `distort` insert. `Soft` is the original soft-clip;
/// the rest are antialiased ADAA shapers (see `dsp/distort.dsp`).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum DistortMode {
    #[default]
    Soft,
    Tanh,
    Arctan,
    Hardclip,
    Parabolic,
    Sinarctan,
}

impl DistortMode {
    /// The `distortmode` slider value the Faust DSP expects.
    pub fn to_index(self) -> f32 {
        match self {
            Self::Soft => 0.0,
            Self::Tanh => 1.0,
            Self::Arctan => 2.0,
            Self::Hardclip => 3.0,
            Self::Parabolic => 4.0,
            Self::Sinarctan => 5.0,
        }
    }
}

impl FromStr for DistortMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "soft" | "0" => Ok(Self::Soft),
            "tanh" | "1" => Ok(Self::Tanh),
            "arctan" | "atan" | "2" => Ok(Self::Arctan),
            "hardclip" | "clip" | "3" => Ok(Self::Hardclip),
            "parabolic" | "para" | "4" => Ok(Self::Parabolic),
            "sinarctan" | "satan" | "5" => Ok(Self::Sinarctan),
            _ => Err(()),
        }
    }
}

/// Fold shape for the `fold` wavefolder insert. `Triangle` is the original
/// reflective fold; see `dsp/fold.dsp`.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum FoldMode {
    #[default]
    Triangle,
    Sine,
    Wrap,
}

impl FoldMode {
    /// The `foldmode` slider value the Faust DSP expects.
    pub fn to_index(self) -> f32 {
        match self {
            Self::Triangle => 0.0,
            Self::Sine => 1.0,
            Self::Wrap => 2.0,
        }
    }
}

impl FromStr for FoldMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "triangle" | "tri" | "0" => Ok(Self::Triangle),
            "sine" | "sin" | "1" => Ok(Self::Sine),
            "wrap" | "2" => Ok(Self::Wrap),
            _ => Err(()),
        }
    }
}

/// Mode for the `flanger` insert. `Classic` is the original undelayed-dry
/// flanger; `ThroughZero` delays the dry by the sweep centre so the notch passes
/// through DC. See `dsp/flanger.dsp`.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum FlangerMode {
    #[default]
    Classic,
    ThroughZero,
}

impl FlangerMode {
    /// The `e_thru` slider value the Faust DSP expects.
    pub fn to_index(self) -> f32 {
        match self {
            Self::Classic => 0.0,
            Self::ThroughZero => 1.0,
        }
    }
}

impl FromStr for FlangerMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "classic" | "0" => Ok(Self::Classic),
            "throughzero" | "thru" | "tzf" | "1" => Ok(Self::ThroughZero),
            _ => Err(()),
        }
    }
}

/// Voicing for the `vinyl` character insert. See `dsp/vinyl.dsp`.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum VinylType {
    #[default]
    Dull,
    Clear,
    Cassette,
}

impl VinylType {
    /// The `e_type` slider value the Faust DSP expects.
    pub fn to_index(self) -> f32 {
        match self {
            Self::Dull => 0.0,
            Self::Clear => 1.0,
            Self::Cassette => 2.0,
        }
    }
}

impl FromStr for VinylType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dull" | "0" => Ok(Self::Dull),
            "clear" | "1" => Ok(Self::Clear),
            "cassette" | "tape" | "2" => Ok(Self::Cassette),
            _ => Err(()),
        }
    }
}

/// Voicing for the `chorus` insert. `Classic` is the original 3-voice chorus;
/// see `dsp/chorus.dsp`.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ChorusType {
    #[default]
    Classic,
    Ensemble,
    Dimension,
}

impl ChorusType {
    /// The `chorustype` slider value the Faust DSP expects.
    pub fn to_index(self) -> f32 {
        match self {
            Self::Classic => 0.0,
            Self::Ensemble => 1.0,
            Self::Dimension => 2.0,
        }
    }
}

impl FromStr for ChorusType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "classic" | "0" => Ok(Self::Classic),
            "ensemble" | "1" => Ok(Self::Ensemble),
            "dimension" | "dim" | "2" => Ok(Self::Dimension),
            _ => Err(()),
        }
    }
}

pub fn midi2freq(note: f32) -> f32 {
    // Clamp the octave exponent far past audibility so a large finite note
    // (note ≳ 1580 overflows to `inf`) stays finite. Mirrors `freq2midi`'s
    // `.max(0.001)` guard on the inverse.
    2.0_f32.powf(((note - 69.0) / 12.0).min(30.0)) * 440.0
}

pub fn freq2midi(freq: f32) -> f32 {
    let safe_freq = freq.max(0.001);
    69.0 + 12.0 * (safe_freq / 440.0).log2()
}
