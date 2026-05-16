//! Bit depth reduction for lo-fi effects.
//!
//! Quantizes amplitude to fewer bits, creating the stepped distortion
//! characteristic of early digital audio. Pair with [`super::coarse`] for
//! full bitcrusher (sample rate + bit depth reduction).

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "crush",
    description: "Bit depth reduction",
    group: ModuleGroup::Effect,
    params: &[ParamInfo {
        name: "crush",
        aliases: &[],
        description: "bit depth (16 = CD, 8 = crunchy, 1 = square wave)",
        default: "0.0",
        min: 0.0,
        max: 16.0,
    }],
};
