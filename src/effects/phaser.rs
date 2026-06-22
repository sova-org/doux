//! Allpass phaser — now Faust-generated; see [`super::faust_dsp`].
//! Only the parameter metadata (`INFO`) remains here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "phaser",
    description: "allpass phaser with feedback resonance",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "phaser",
            aliases: &["phaserrate"],
            description: "LFO rate in Hz (0 = bypass)",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "phaserdepth",
            aliases: &[],
            description: "feedback resonance",
            default: "0.75",
            min: 0.0,
            max: 0.95,
        },
        ParamInfo {
            name: "phasersweep",
            aliases: &[],
            description: "modulation range in cents",
            default: "1200.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "phasercenter",
            aliases: &[],
            description: "base center frequency in Hz",
            default: "800.0",
            min: 0.0,
            max: 20000.0,
        },
    ],
};
