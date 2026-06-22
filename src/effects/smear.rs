//! Allpass-cascade smear — now Faust-generated; see [`super::faust_dsp`].
//! Only the parameter metadata (`INFO`) remains here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "smear",
    description: "12-stage cascaded allpass chain",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "smear",
            aliases: &[],
            description: "wet/dry mix (0 = bypass, 1 = full wet)",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "smearfreq",
            aliases: &[],
            description: "allpass break frequency in Hz",
            default: "1000.0",
            min: 20.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "smearfb",
            aliases: &[],
            description: "feedback for resonance",
            default: "0.0",
            min: 0.0,
            max: 0.95,
        },
    ],
};
