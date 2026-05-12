//! Per-orbit stereo feedback delay.
//!
//! Re-injects the voice's output back into itself with a controllable delay
//! time, damping, and cross-channel blend. Enables slapback echoes, metallic
//! resonances, ping-pong, and short rhythmic feedback loops.

use crate::dsp::{ftz, ms_to_samples, Phasor};
use crate::types::{LfoShape, ModuleGroup, ModuleInfo, ParamInfo, CHANNELS};

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
const BUFFER_MASK: usize = BUFFER_SIZE - 1;

#[derive(Clone, Copy)]
pub struct FeedbackParams {
    pub time_ms: f32,
    pub damp: f32,
    pub cross: f32,
    pub lfo: f32,
    pub lfo_depth: f32,
    pub lfo_shape: LfoShape,
}

impl Default for FeedbackParams {
    fn default() -> Self {
        Self {
            time_ms: 10.0,
            damp: 0.0,
            cross: 0.0,
            lfo: 0.0,
            lfo_depth: 0.5,
            lfo_shape: LfoShape::Sine,
        }
    }
}

/// Stereo feedback delay with one-pole damping and cross-channel blend.
#[derive(Clone)]
pub struct Feedback {
    buffer: [Vec<f32>; CHANNELS],
    write_pos: [usize; CHANNELS],
    damp_state: [f32; CHANNELS],
    phasor: Phasor,
    pub params: FeedbackParams,
}

impl Default for Feedback {
    fn default() -> Self {
        Self {
            buffer: [vec![0.0; BUFFER_SIZE], vec![0.0; BUFFER_SIZE]],
            write_pos: [0; CHANNELS],
            damp_state: [0.0; CHANNELS],
            phasor: Phasor::default(),
            params: FeedbackParams::default(),
        }
    }
}

impl Feedback {
    #[inline]
    fn read(&self, ch: usize, delay_samples: f32) -> f32 {
        let delay_int = delay_samples.floor() as usize;
        let frac = delay_samples - delay_int as f32;
        let buf = &self.buffer[ch];
        let wp = self.write_pos[ch];
        let idx0 = (wp + BUFFER_SIZE - delay_int) & BUFFER_MASK;
        let idx1 = (wp + BUFFER_SIZE - delay_int - 1) & BUFFER_MASK;
        buf[idx0] + frac * (buf[idx1] - buf[idx0])
    }

    #[inline]
    fn write(&mut self, ch: usize, sample: f32) {
        let buf = &mut self.buffer[ch];
        let wp = self.write_pos[ch];
        buf[wp] = sample;
        self.write_pos[ch] = (wp + 1) & BUFFER_MASK;
    }

    /// Processes one stereo sample through the feedback delay.
    ///
    /// `fb_amount` is the re-injection coefficient supplied by the caller
    /// (typically the orbit's send level, which doubles as the FX feedback
    /// gain). Time/damp/cross and LFO modulation come from `self.params`.
    ///
    /// Returns wet signal only (dry is summed separately by the orbit bus).
    pub fn process(
        &mut self,
        input: [f32; CHANNELS],
        fb_amount: f32,
        sr: f32,
    ) -> [f32; CHANNELS] {
        let p = self.params;
        let isr = 1.0 / sr;
        let time_ms = if p.lfo > 0.0 {
            let lfo = self.phasor.lfo(p.lfo_shape, p.lfo, isr);
            p.time_ms + lfo * p.lfo_depth * p.time_ms * 0.5
        } else {
            p.time_ms
        };

        let delay_samples = ms_to_samples(time_ms, sr).clamp(1.0, (BUFFER_SIZE - 1) as f32);
        let fb_amount = fb_amount.clamp(0.0, 0.99);
        let cross = p.cross.clamp(0.0, 1.0);

        let mut delayed = [0.0f32; CHANNELS];
        let mut damped = [0.0f32; CHANNELS];

        for c in 0..CHANNELS {
            delayed[c] = self.read(c, delay_samples);
            damped[c] = if p.damp > 0.0 {
                self.damp_state[c] = ftz(
                    delayed[c] * (1.0 - p.damp) + self.damp_state[c] * p.damp,
                    0.0001,
                );
                self.damp_state[c]
            } else {
                delayed[c]
            };
        }

        let fb_l = damped[0] * (1.0 - cross) + damped[1] * cross;
        let fb_r = damped[1] * (1.0 - cross) + damped[0] * cross;

        self.write(0, input[0] + fb_l * fb_amount);
        self.write(1, input[1] + fb_r * fb_amount);

        delayed
    }
}
