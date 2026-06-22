//! Delay metadata + params. The DSP now lives in `dsp/delay.dsp`
//! (`effects::FaustDelay`); this module keeps the registry `INFO` and the
//! `DelayParams` (incl. `DelayType`) the orbit threads into the Faust wrapper.

use crate::types::{DelayType, ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "delay",
    description: "Delay with multiple algorithms (standard, pingpong, tape, multitap)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "delay",
            aliases: &[],
            description: "send level",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "delaytime",
            aliases: &[],
            description: "time in seconds",
            default: "0.333",
            min: 0.0,
            max: 10.0,
        },
        ParamInfo {
            name: "delayfeedback",
            aliases: &[],
            description: "feedback amount",
            default: "0.6",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "delaytype",
            aliases: &["dtype"],
            description: "algorithm (standard, pingpong, tape, multitap)",
            default: "0.0",
            min: 0.0,
            max: 3.0,
        },
    ],
};

#[derive(Clone, Copy)]
pub struct DelayParams {
    pub time: f32,
    pub feedback: f32,
    pub delay_type: DelayType,
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            time: 0.333,
            feedback: 0.6,
            delay_type: DelayType::Standard,
        }
    }
}
