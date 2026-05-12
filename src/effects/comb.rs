//! Comb filter with damping.
//!
//! Creates resonant peaks at `freq` and its harmonics by feeding delayed
//! signal back into itself. Damping applies a lowpass in the feedback path,
//! causing higher harmonics to decay faster (Karplus-Strong style).

use crate::dsp::DelayLine;
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

const BUFFER_SIZE: usize = 2048;

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

/// Feedback comb filter with one-pole damping.
#[derive(Clone, Copy, Default)]
pub struct Comb {
    delay: DelayLine<BUFFER_SIZE>,
    damp_state: f32,
}

impl Comb {
    /// Processes one sample through the comb filter. Params are shared
    /// across channels and supplied by the caller (typically the orbit).
    pub fn process(&mut self, input: f32, p: &CombParams, sr: f32) -> f32 {
        let delay_samples = (sr / p.freq).clamp(1.0, (BUFFER_SIZE - 1) as f32);
        let delayed = self.delay.read(delay_samples);

        let feedback = p.feedback.clamp(-0.99, 0.99);
        let fb_signal = if p.damp > 0.0 {
            self.damp_state = delayed * (1.0 - p.damp) + self.damp_state * p.damp;
            self.damp_state
        } else {
            delayed
        };

        self.delay.write(input + fb_signal * feedback);
        delayed
    }
}
