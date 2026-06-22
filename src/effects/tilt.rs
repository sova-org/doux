//! Single-knob tilt EQ — now Faust-generated; see [`super::faust_dsp`].
//! Only the parameter metadata (`INFO`) remains here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "tilt",
    description: "Single-knob spectral balance tilt EQ",
    group: ModuleGroup::Effect,
    params: &[ParamInfo {
        name: "tilt",
        aliases: &[],
        description: "spectral balance (-1 dark, 0 flat, 1 bright)",
        default: "0.0",
        min: -1.0,
        max: 1.0,
    }],
};
