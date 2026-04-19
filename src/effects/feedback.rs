//! Per-voice feedback delay.
//!
//! Re-injects the voice's output back into itself with a controllable delay
//! time and damping. Enables slapback echoes, metallic resonances, and
//! short rhythmic feedback loops.

use crate::dsp::DelayLine;
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
            max: 500.0,
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
            name: "fblfo",
            aliases: &[],
            description: "LFO rate in Hz",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "fblfodepth",
            aliases: &[],
            description: "LFO depth",
            default: "0.5",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "fblfoshape",
            aliases: &[],
            description: "LFO waveform (sine, tri, saw, square, sh)",
            default: "sine",
            min: 0.0,
            max: 0.0,
        },
    ],
};

const BUFFER_SIZE: usize = 32768;

/// Feedback delay with one-pole damping in the feedback path.
#[derive(Clone, Copy, Default)]
pub struct Feedback {
    delay: DelayLine<BUFFER_SIZE>,
    damp_state: f32,
}

impl Feedback {
    /// Processes one sample through the feedback delay.
    ///
    /// - `feedback`: Re-injection level `[0.0, 0.99]`
    /// - `time_ms`: Delay time in milliseconds
    /// - `damp`: High-frequency loss in feedback path `[0.0, 1.0]`
    ///
    /// Returns 50/50 dry/wet mix.
    pub fn process(&mut self, input: f32, feedback: f32, time_ms: f32, damp: f32, sr: f32) -> f32 {
        let delay_samples = (time_ms * sr * 0.001).clamp(1.0, (BUFFER_SIZE - 1) as f32);
        let delayed = self.delay.read(delay_samples);

        let feedback = feedback.clamp(0.0, 0.99);
        let fb_signal = if damp > 0.0 {
            self.damp_state = delayed * (1.0 - damp) + self.damp_state * damp;
            self.damp_state
        } else {
            delayed
        };

        self.delay.write(input + fb_signal * feedback);
        input * 0.5 + delayed * 0.5
    }
}
