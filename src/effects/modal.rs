//! Modal resonator — the bank itself lives in [`arf::modal`], shared with the `modal` UGen
//! so a patch and a param run the same eight modes. Only the parameter metadata (`INFO`)
//! lives here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "modal",
    description: "Modal resonator: eight tuned modes rung by the voice, string to bar to bell",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "modal",
            aliases: &[],
            description: "dry/wet mix (0 = bypass)",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "modalfreq",
            aliases: &[],
            description: "fundamental in Hz — the pitch the bank rings at",
            default: "220.0",
            min: 20.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "modaldecay",
            aliases: &[],
            description: "ring time of mode 1 in seconds",
            default: "2.0",
            min: 0.05,
            max: 20.0,
        },
        ParamInfo {
            name: "modalstruct",
            aliases: &[],
            description: "partial ratios: 0 = string, 0.5 = bar, 1 = bell",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "modalbright",
            aliases: &[],
            description: "how long the upper modes ring relative to mode 1",
            default: "0.5",
            min: 0.0,
            max: 1.0,
        },
    ],
};
