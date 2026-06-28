//! Single-sideband frequency shifter — Faust-generated; see [`super::faust_dsp`].
//! Only the parameter metadata (`INFO`) remains here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "fshift",
    description: "single-sideband frequency shifter (inharmonic; not a transpose)",
    group: ModuleGroup::Effect,
    params: &[ParamInfo {
        name: "fshift",
        aliases: &["fsh"],
        description: "shift in Hz; positive shifts up, negative down (0 = bypass)",
        default: "0.0",
        min: -2000.0,
        max: 2000.0,
    }],
};
