//! VinylSim / Cassette character insert — Faust-generated; see [`super::faust_dsp`].
//! Only the parameter metadata (`INFO`) lives here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "vinyl",
    description: "Vinyl / cassette character: wow+flutter, band-limit, hiss, saturation",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "vinyl",
            aliases: &[],
            description: "dry/wet mix (0 = bypass)",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "vinylwow",
            aliases: &[],
            description: "wow + flutter depth",
            default: "0.3",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "vinylnoise",
            aliases: &[],
            description: "hiss level",
            default: "0.2",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "vinyltone",
            aliases: &[],
            description: "tone tilt (-1 darker .. 1 brighter)",
            default: "0.0",
            min: -1.0,
            max: 1.0,
        },
        ParamInfo {
            name: "vinyltype",
            aliases: &[],
            description: "voicing (vinyl303, vinyl404, cassette)",
            default: "vinyl303",
            min: 0.0,
            max: 2.0,
        },
    ],
};
