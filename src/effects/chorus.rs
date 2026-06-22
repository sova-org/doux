//! Modulated-delay chorus — now Faust-generated; see [`super::faust_dsp`].
//! Only the parameter metadata (`INFO`) remains here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "chorus",
    description: "Modulated delay with stereo spread (classic / ensemble / dimension)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "chorus",
            aliases: &["chorusrate"],
            description: "LFO rate in Hz (0 = bypass)",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "chorusdepth",
            aliases: &[],
            description: "modulation intensity",
            default: "0.35",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "chorusdelay",
            aliases: &[],
            description: "base delay time in ms",
            default: "25.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "chorustype",
            aliases: &["ctype"],
            description: "voicing (classic, ensemble, dimension)",
            default: "classic",
            min: 0.0,
            max: 2.0,
        },
    ],
};
