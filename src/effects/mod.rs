mod chorus;
mod coarse;
mod comb;
mod compressor;
mod crush;
mod delay;
mod distort;
mod eq;
mod feedback;
mod flanger;
mod haas;
mod ladder;
mod lag;
mod phaser;
mod reverb;
mod smear;
mod tilt;
mod vital_reverb;

pub use chorus::Chorus;
pub use coarse::Coarse;
pub use comb::Comb;
pub use compressor::Compressor;
pub use crush::crush;
pub use delay::{Delay, DelayParams};
pub use distort::{distort, DcBlocker, Fold, Wrap};
pub use eq::Eq;
pub use feedback::Feedback;
pub use flanger::Flanger;
pub use haas::Haas;
pub use ladder::{LadderFilter, LadderMode};
pub use lag::Lag;
pub use phaser::Phaser;
pub use reverb::DattorroVerb;
pub use smear::Smear;
pub use tilt::Tilt;
pub use vital_reverb::VitalVerb;

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

// ── Synthesis ────────────────────────────────────────────────────────────

const INFO_PITCH: ModuleInfo = ModuleInfo {
    name: "pitch",
    description: "Pitch and frequency control",
    group: ModuleGroup::Synthesis,
    params: &[
        ParamInfo {
            name: "freq",
            aliases: &[],
            description: "base frequency in Hz",
            default: "330.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "note",
            aliases: &[],
            description: "MIDI note number (converted to freq)",
            default: "0.0",
            min: 0.0,
            max: 127.0,
        },
        ParamInfo {
            name: "detune",
            aliases: &[],
            description: "pitch offset in cents",
            default: "0.0",
            min: -1200.0,
            max: 1200.0,
        },
        ParamInfo {
            name: "speed",
            aliases: &[],
            description: "playback speed multiplier",
            default: "1.0",
            min: -100.0,
            max: 100.0,
        },
    ],
};

const INFO_TIMING: ModuleInfo = ModuleInfo {
    name: "timing",
    description: "Timing and gate control",
    group: ModuleGroup::Synthesis,
    params: &[
        ParamInfo {
            name: "tick",
            aliases: &[],
            description: "trigger time in samples",
            default: "0.0",
            min: 0.0,
            max: f32::MAX,
        },
        ParamInfo {
            name: "gate",
            aliases: &[],
            description: "gate duration in seconds",
            default: "1.0",
            min: 0.0,
            max: f32::MAX,
        },
    ],
};

const INFO_VOICE: ModuleInfo = ModuleInfo {
    name: "voice",
    description: "Voice allocation and routing",
    group: ModuleGroup::Synthesis,
    params: &[
        ParamInfo {
            name: "voice",
            aliases: &[],
            description: "voice index (fixed allocation)",
            default: "0.0",
            min: 0.0,
            max: 31.0,
        },
        ParamInfo {
            name: "orbit",
            aliases: &[],
            description: "effect bus index",
            default: "0.0",
            min: 0.0,
            max: 7.0,
        },
        ParamInfo {
            name: "reset",
            aliases: &[],
            description: "reset voice state",
            default: "0",
            min: 0.0,
            max: 0.0,
        },
    ],
};

const INFO_GAIN: ModuleInfo = ModuleInfo {
    name: "gain",
    description: "Gain, panning, and stereo control",
    group: ModuleGroup::Synthesis,
    params: &[
        ParamInfo {
            name: "gain",
            aliases: &[],
            description: "pre-filter gain",
            default: "1.0",
            min: 0.0,
            max: 10.0,
        },
        ParamInfo {
            name: "postgain",
            aliases: &[],
            description: "post-envelope gain",
            default: "1.0",
            min: 0.0,
            max: 10.0,
        },
        ParamInfo {
            name: "velocity",
            aliases: &[],
            description: "MIDI velocity multiplier",
            default: "1.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "pan",
            aliases: &[],
            description: "stereo pan (0 left, 0.5 center, 1 right)",
            default: "0.5",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "width",
            aliases: &[],
            description: "stereo width (0 mono, 1 unchanged, 2 exaggerated)",
            default: "1.0",
            min: 0.0,
            max: 2.0,
        },
        ParamInfo {
            name: "haas",
            aliases: &[],
            description: "stereo placement delay in ms",
            default: "0.0",
            min: 0.0,
            max: 30.0,
        },
    ],
};

const INFO_OSCILLATOR: ModuleInfo = ModuleInfo {
    name: "oscillator",
    description: "Oscillator shape and modifiers",
    group: ModuleGroup::Synthesis,
    params: &[
        ParamInfo {
            name: "pw",
            aliases: &[],
            description: "pulse width",
            default: "0.5",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "spread",
            aliases: &[],
            description: "unison spread in cents",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "size",
            aliases: &[],
            description: "phase quantization steps",
            default: "0.0",
            min: 0.0,
            max: 256.0,
        },
        ParamInfo {
            name: "warp",
            aliases: &[],
            description: "phase warp distortion",
            default: "0.0",
            min: -1.0,
            max: 1.0,
        },
        ParamInfo {
            name: "mirror",
            aliases: &[],
            description: "phase mirror position",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "sub",
            aliases: &[],
            description: "sub oscillator mix level",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "suboct",
            aliases: &[],
            description: "sub oscillator octave (1-3)",
            default: "1.0",
            min: 1.0,
            max: 3.0,
        },
        ParamInfo {
            name: "subwave",
            aliases: &[],
            description: "sub oscillator waveform (tri, sine, square)",
            default: "tri",
            min: 0.0,
            max: 0.0,
        },
    ],
};

const INFO_ENVELOPE: ModuleInfo = ModuleInfo {
    name: "envelope",
    description: "DAHDSR amplitude envelope",
    group: ModuleGroup::Synthesis,
    params: &[
        ParamInfo {
            name: "envdelay",
            aliases: &["envdly"],
            description: "delay time in seconds",
            default: "0.0",
            min: 0.0,
            max: 10.0,
        },
        ParamInfo {
            name: "attack",
            aliases: &[],
            description: "attack time in seconds",
            default: "0.003",
            min: 0.0,
            max: 10.0,
        },
        ParamInfo {
            name: "hold",
            aliases: &["hld"],
            description: "hold time at peak in seconds",
            default: "0.0",
            min: 0.0,
            max: 10.0,
        },
        ParamInfo {
            name: "decay",
            aliases: &[],
            description: "decay time in seconds",
            default: "0.0",
            min: 0.0,
            max: 10.0,
        },
        ParamInfo {
            name: "sustain",
            aliases: &[],
            description: "sustain level",
            default: "1.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "release",
            aliases: &[],
            description: "release time in seconds",
            default: "0.005",
            min: 0.0,
            max: 10.0,
        },
    ],
};

const INFO_VIBRATO: ModuleInfo = ModuleInfo {
    name: "vibrato",
    description: "Pitch LFO modulation",
    group: ModuleGroup::Synthesis,
    params: &[
        ParamInfo {
            name: "vib",
            aliases: &[],
            description: "LFO rate in Hz",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "vibmod",
            aliases: &[],
            description: "depth in semitones",
            default: "0.5",
            min: 0.0,
            max: 12.0,
        },
        ParamInfo {
            name: "vibshape",
            aliases: &[],
            description: "LFO waveform (sine, tri, saw, square, sh)",
            default: "sine",
            min: 0.0,
            max: 0.0,
        },
    ],
};

const INFO_FM: ModuleInfo = ModuleInfo {
    name: "fm",
    description: "Frequency modulation synthesis",
    group: ModuleGroup::Synthesis,
    params: &[
        ParamInfo {
            name: "fm",
            aliases: &["fmi"],
            description: "modulation index (depth)",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "fmh",
            aliases: &[],
            description: "harmonic ratio",
            default: "1.0",
            min: 0.0,
            max: 32.0,
        },
        ParamInfo {
            name: "fm2",
            aliases: &[],
            description: "operator 2 modulation index",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "fm2h",
            aliases: &[],
            description: "operator 2 harmonic ratio",
            default: "1.0",
            min: 0.0,
            max: 32.0,
        },
        ParamInfo {
            name: "fmalgo",
            aliases: &[],
            description: "algorithm (0=cascade, 1=parallel, 2=branch)",
            default: "0.0",
            min: 0.0,
            max: 2.0,
        },
        ParamInfo {
            name: "fmfb",
            aliases: &[],
            description: "feedback on topmost operator",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "fmshape",
            aliases: &[],
            description: "modulator waveform (sine, tri, saw, square, sh)",
            default: "sine",
            min: 0.0,
            max: 0.0,
        },
    ],
};

const INFO_AM: ModuleInfo = ModuleInfo {
    name: "am",
    description: "Amplitude modulation",
    group: ModuleGroup::Synthesis,
    params: &[
        ParamInfo {
            name: "am",
            aliases: &[],
            description: "LFO rate in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "amdepth",
            aliases: &[],
            description: "modulation depth",
            default: "0.5",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "amshape",
            aliases: &[],
            description: "LFO waveform (sine, tri, saw, square, sh)",
            default: "sine",
            min: 0.0,
            max: 0.0,
        },
    ],
};

const INFO_RM: ModuleInfo = ModuleInfo {
    name: "rm",
    description: "Ring modulation",
    group: ModuleGroup::Synthesis,
    params: &[
        ParamInfo {
            name: "rm",
            aliases: &[],
            description: "modulator frequency in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "rmdepth",
            aliases: &[],
            description: "modulation depth",
            default: "1.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "rmshape",
            aliases: &[],
            description: "modulator waveform (sine, tri, saw, square, sh)",
            default: "sine",
            min: 0.0,
            max: 0.0,
        },
    ],
};

const INFO_RECORDER: ModuleInfo = ModuleInfo {
    name: "recorder",
    description: "Audio recording and overdubbing",
    group: ModuleGroup::Synthesis,
    params: &[ParamInfo {
        name: "overdub",
        aliases: &["dub"],
        description: "layer on existing recording",
        default: "false",
        min: 0.0,
        max: 0.0,
    }],
};

// ── Filters (Effect group) ──────────────────────────────────────────────

const INFO_LPF: ModuleInfo = ModuleInfo {
    name: "lpf",
    description: "State variable lowpass filter",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "lpf",
            aliases: &["cutoff"],
            description: "cutoff frequency in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "lpq",
            aliases: &["resonance"],
            description: "resonance",
            default: "0.2",
            min: 0.0,
            max: 1.0,
        },
    ],
};

const INFO_HPF: ModuleInfo = ModuleInfo {
    name: "hpf",
    description: "State variable highpass filter",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "hpf",
            aliases: &["hcutoff"],
            description: "cutoff frequency in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "hpq",
            aliases: &["hresonance"],
            description: "resonance",
            default: "0.2",
            min: 0.0,
            max: 1.0,
        },
    ],
};

const INFO_BPF: ModuleInfo = ModuleInfo {
    name: "bpf",
    description: "State variable bandpass filter",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "bpf",
            aliases: &["bandf"],
            description: "center frequency in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "bpq",
            aliases: &["bandq"],
            description: "resonance",
            default: "0.2",
            min: 0.0,
            max: 1.0,
        },
    ],
};

// ── Registry ────────────────────────────────────────────────────────────

pub const ALL_MODULES: &[&ModuleInfo] = &[
    // Synthesis
    &INFO_PITCH,
    &INFO_TIMING,
    &INFO_VOICE,
    &INFO_GAIN,
    &INFO_OSCILLATOR,
    &INFO_ENVELOPE,
    &INFO_VIBRATO,
    &INFO_FM,
    &INFO_AM,
    &INFO_RM,
    &INFO_RECORDER,
    // Effects — filters
    &INFO_LPF,
    &INFO_HPF,
    &INFO_BPF,
    &ladder::INFO_LLPF,
    &ladder::INFO_LHPF,
    &ladder::INFO_LBPF,
    // Effects — modulation
    &phaser::INFO,
    &flanger::INFO,
    &chorus::INFO,
    &smear::INFO,
    // Effects — distortion
    &coarse::INFO,
    &crush::INFO,
    &distort::INFO,
    // Effects — EQ
    &eq::INFO,
    &tilt::INFO,
    // Effects — spatial/send
    &delay::INFO,
    &reverb::INFO,
    &comb::INFO,
    &feedback::INFO,
    &compressor::INFO,
];
