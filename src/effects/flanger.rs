//! Flanger effect with LFO-modulated delay.
//!
//! Creates the characteristic "jet plane" sweep by mixing the input with a
//! short, modulated delay (0.5-10ms). Feedback intensifies the comb filtering.

use crate::dsp::{ms_to_samples, DelayLine, Phasor};
use crate::types::{ModuleGroup, ModuleInfo, ParamInfo, StereoFrame};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "flanger",
    description: "LFO-modulated short delay with feedback",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "flanger",
            aliases: &["flangerrate"],
            description: "LFO rate in Hz (0 = bypass)",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "flangerdepth",
            aliases: &[],
            description: "modulation depth",
            default: "0.7",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "flangerfeedback",
            aliases: &[],
            description: "feedback amount",
            default: "0.35",
            min: 0.0,
            max: 0.95,
        },
    ],
};

const BUFFER_SIZE: usize = 512;
const MIN_DELAY_MS: f32 = 0.5;
const MAX_DELAY_MS: f32 = 10.0;
const DELAY_RANGE_MS: f32 = MAX_DELAY_MS - MIN_DELAY_MS;

/// Mono flanger with feedback.
#[derive(Clone, Copy, Default)]
pub struct Flanger {
    delay: DelayLine<BUFFER_SIZE>,
    lfo: Phasor,
    feedback_sample: f32,
}

impl Flanger {
    /// Processes a block of stereo frames in place on channel `ch`.
    ///
    /// `depth_curve`, `feedback` clamp, and `delay_span_ms` hoist to block entry;
    /// LFO ticks per sample and feedback path stays per-sample.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        rate: f32,
        depth: f32,
        feedback: f32,
        sr: f32,
        isr: f32,
    ) {
        let depth_curve = depth * depth;
        let span = depth_curve * DELAY_RANGE_MS;
        let feedback = feedback.clamp(0.0, 0.95);
        let max_delay_samples = BUFFER_SIZE as f32 - 2.0;
        for slot in buf.iter_mut().take(n) {
            let lfo_val = self.lfo.sine(rate, isr);
            let delay_ms = MIN_DELAY_MS + span * (lfo_val * 0.5 + 0.5);
            let delay_samples = ms_to_samples(delay_ms, sr).clamp(1.0, max_delay_samples);
            let delayed = self.delay.read(delay_samples);
            let input = slot[ch];
            self.delay.write(input + self.feedback_sample * feedback);
            self.feedback_sample = delayed;
            slot[ch] = input * 0.5 + delayed * 0.5;
        }
    }
}
