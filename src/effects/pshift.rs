//! Granular pitch shifter — Faust-generated (`ef.transpose`); see [`super::faust_dsp`].
//! Only the parameter metadata (`INFO`) remains here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "pshift",
    description: "granular pitch shifter — transposes in semitones (preserves harmony)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "pshift",
            aliases: &["psh"],
            description: "transposition in semitones; signed (0 = bypass)",
            default: "0.0",
            min: -24.0,
            max: 24.0,
        },
        ParamInfo {
            name: "pshiftwin",
            aliases: &["pwin"],
            description: "grain window in ms (small = grainy, large = smoother)",
            default: "40.0",
            min: 5.0,
            max: 200.0,
        },
    ],
};
