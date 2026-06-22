//! Moog-style ladder filter — now Faust-generated; see `super::faust_dsp`
//! (`FaustLadder`). The parameter metadata (`INFO_*`) and the [`LadderMode`]
//! selector remain here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO_LLPF: ModuleInfo = ModuleInfo {
    name: "llpf",
    description: "Moog-style ladder lowpass filter",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "llpf",
            aliases: &[],
            description: "cutoff frequency in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "llpq",
            aliases: &[],
            description: "resonance",
            default: "0.2",
            min: 0.0,
            max: 1.0,
        },
    ],
};

pub const INFO_LHPF: ModuleInfo = ModuleInfo {
    name: "lhpf",
    description: "Moog-style ladder highpass filter",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "lhpf",
            aliases: &[],
            description: "cutoff frequency in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "lhpq",
            aliases: &[],
            description: "resonance",
            default: "0.2",
            min: 0.0,
            max: 1.0,
        },
    ],
};

pub const INFO_LBPF: ModuleInfo = ModuleInfo {
    name: "lbpf",
    description: "Moog-style ladder bandpass filter",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "lbpf",
            aliases: &[],
            description: "cutoff frequency in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "lbpq",
            aliases: &[],
            description: "resonance",
            default: "0.2",
            min: 0.0,
            max: 1.0,
        },
    ],
};

/// Multimode selector for the ladder filter.
#[derive(Clone, Copy, PartialEq)]
pub enum LadderMode {
    Lp,
    Hp,
    Bp,
}
