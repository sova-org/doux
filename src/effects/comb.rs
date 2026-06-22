//! Comb filter metadata + params.
//!
//! Creates resonant peaks at `freq` and its harmonics by feeding delayed
//! signal back into itself, with a lowpass in the feedback path (Karplus-Strong
//! style). The DSP now lives in `dsp/comb.dsp` (`effects::FaustComb`); this
//! module keeps the registry `INFO` and the `CombParams` the orbit threads into
//! the Faust wrapper.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "comb",
    description: "Feedback comb filter with damping (Karplus-Strong style)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "comb",
            aliases: &[],
            description: "send level",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "combfreq",
            aliases: &[],
            description: "fundamental frequency in Hz",
            default: "220.0",
            min: 20.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "combfeedback",
            aliases: &[],
            description: "feedback amount",
            default: "0.9",
            min: -0.99,
            max: 0.99,
        },
        ParamInfo {
            name: "combdamp",
            aliases: &[],
            description: "high-frequency damping",
            default: "0.1",
            min: 0.0,
            max: 1.0,
        },
    ],
};

#[derive(Clone, Copy)]
pub struct CombParams {
    pub freq: f32,
    pub feedback: f32,
    pub damp: f32,
}

impl Default for CombParams {
    fn default() -> Self {
        Self {
            freq: 220.0,
            feedback: 0.9,
            damp: 0.1,
        }
    }
}
