//! Per-orbit stereo feedback delay metadata + params.
//!
//! Re-injects the bus signal into itself with a controllable delay time,
//! damping, and cross-channel blend (slapback, metallic resonance, ping-pong).
//! The DSP now lives in `dsp/feedback.dsp` (`effects::FaustFeedback`); this
//! module keeps the registry `INFO` and the `FeedbackParams` the orbit threads
//! into the Faust wrapper.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "feedback",
    description: "Per-voice re-injection delay",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "feedback",
            aliases: &["fb"],
            description: "send / re-injection amount",
            default: "0.0",
            min: 0.0,
            max: 0.99,
        },
        ParamInfo {
            name: "fbtime",
            aliases: &["fbt"],
            description: "delay time in ms",
            default: "10.0",
            min: 0.0,
            max: 680.0,
        },
        ParamInfo {
            name: "fbdamp",
            aliases: &["fbd"],
            description: "damping in feedback path",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "fbcross",
            aliases: &["fbc"],
            description: "cross-channel blend (0 = self, 1 = ping-pong)",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
    ],
};

#[derive(Clone, Copy)]
pub struct FeedbackParams {
    pub time_ms: f32,
    pub damp: f32,
    pub cross: f32,
}

impl Default for FeedbackParams {
    fn default() -> Self {
        Self {
            time_ms: 10.0,
            damp: 0.0,
            cross: 0.0,
        }
    }
}
