//! LFO-modulated flanger — now Faust-generated; see [`super::faust_dsp`].
//! Only the parameter metadata (`INFO`) remains here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "flanger",
    description: "LFO-modulated short delay with feedback",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "flanger",
            aliases: &["flangerrate"],
            description: "LFO rate in Hz (0 = bypass)",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "flangerdepth",
            aliases: &[],
            description: "modulation depth",
            default: "0.7",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "flangerfeedback",
            aliases: &[],
            description: "feedback amount",
            default: "0.35",
            min: 0.0,
            max: 0.95,
        },
        ParamInfo {
            name: "flangermode",
            aliases: &["flmode"],
            description: "mode (classic, throughzero)",
            default: "classic",
            min: 0.0,
            max: 1.0,
        },
    ],
};
