//! 3-band DJ-style EQ — now Faust-generated; see [`super::faust_dsp`].
//! Only the parameter metadata (`INFO`) remains here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "eq",
    description: "3-band parametric EQ (low shelf, mid peak, high shelf)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "eqlo",
            aliases: &[],
            description: "low shelf gain in dB",
            default: "0.0",
            min: -24.0,
            max: 24.0,
        },
        ParamInfo {
            name: "eqmid",
            aliases: &[],
            description: "mid peak gain in dB",
            default: "0.0",
            min: -24.0,
            max: 24.0,
        },
        ParamInfo {
            name: "eqhi",
            aliases: &[],
            description: "high shelf gain in dB",
            default: "0.0",
            min: -24.0,
            max: 24.0,
        },
        ParamInfo {
            name: "eqlofreq",
            aliases: &[],
            description: "low shelf frequency in Hz",
            default: "200.0",
            min: 20.0,
            max: 2000.0,
        },
        ParamInfo {
            name: "eqmidfreq",
            aliases: &[],
            description: "mid peak frequency in Hz",
            default: "1000.0",
            min: 100.0,
            max: 10000.0,
        },
        ParamInfo {
            name: "eqmidq",
            aliases: &[],
            description: "mid peak Q / bandwidth (0.7 = original)",
            default: "0.7",
            min: 0.2,
            max: 8.0,
        },
        ParamInfo {
            name: "eqhifreq",
            aliases: &[],
            description: "high shelf frequency in Hz",
            default: "5000.0",
            min: 1000.0,
            max: 20000.0,
        },
    ],
};
