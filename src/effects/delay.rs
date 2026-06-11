use crate::dsp::ftz;
use crate::types::{DelayType, ModuleGroup, ModuleInfo, ParamInfo, StereoFrame, CHANNELS};

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

const MAX_DELAY_SAMPLES: usize = 65536;

#[derive(Clone)]
struct DelayLine {
    buffer: Vec<f32>,
    mask: usize,
    write_pos: usize,
}

impl DelayLine {
    fn new(max_samples: usize) -> Self {
        debug_assert!(max_samples.is_power_of_two());
        Self {
            buffer: vec![0.0; max_samples],
            mask: max_samples - 1,
            write_pos: 0,
        }
    }

    fn process(&mut self, input: f32, delay_samples: f32) -> f32 {
        self.buffer[self.write_pos] = input;
        let out = self.read_at(delay_samples);
        self.write_pos = (self.write_pos + 1) & self.mask;
        out
    }

    /// Fractional read with linear interpolation (same scheme as
    /// `Feedback::read`), so a moving delay time glides instead of stepping.
    fn read_at(&self, delay_samples: f32) -> f32 {
        let d = delay_samples.clamp(0.0, (self.mask - 1) as f32);
        let di = d as usize;
        let frac = d - di as f32;
        let i0 = self.write_pos.wrapping_sub(di) & self.mask;
        let i1 = self.write_pos.wrapping_sub(di + 1) & self.mask;
        let y0 = self.buffer[i0];
        let y1 = self.buffer[i1];
        y0 + frac * (y1 - y0)
    }

    fn write(&mut self, input: f32) {
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) & self.mask;
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
    }
}

impl Default for DelayLine {
    fn default() -> Self {
        Self::new(MAX_DELAY_SAMPLES)
    }
}

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

/// One-pole time constant for delay-time changes. Converges in ~150ms —
/// well inside the orbit's 1s silence holdoff, so an idle orbit's delay
/// has settled before its FX are bypassed.
const TIME_SLEW_SECS: f32 = 0.03;

#[derive(Clone)]
pub struct Delay {
    lines: [DelayLine; CHANNELS],
    feedback: [f32; CHANNELS],
    lp: [f32; CHANNELS],
    /// Smoothed delay time in samples; negative = snap to target on next use,
    /// so the first echo after construction/clear never glides.
    time_smooth: f32,
    time_slew: f32,
    pub params: DelayParams,
    sr: f32,
}

impl Delay {
    pub fn new(sr: f32) -> Self {
        Self {
            lines: Default::default(),
            feedback: [0.0; CHANNELS],
            lp: [0.0; CHANNELS],
            time_smooth: -1.0,
            time_slew: 1.0 - (-1.0 / (TIME_SLEW_SECS * sr)).exp(),
            params: DelayParams::default(),
            sr,
        }
    }

    pub fn process(&mut self, send: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let p = self.params;
        let target = (p.time * self.sr).clamp(0.0, (MAX_DELAY_SAMPLES - 2) as f32);
        if self.time_smooth < 0.0 {
            self.time_smooth = target;
        } else {
            self.time_smooth += self.time_slew * (target - self.time_smooth);
        }
        let delay_samples = self.time_smooth;
        let feedback = p.feedback.clamp(0.0, 0.95);

        match p.delay_type {
            DelayType::Standard => {
                let mut out = [0.0; CHANNELS];
                for c in 0..CHANNELS {
                    let fb = ftz(self.feedback[c], 0.0001);
                    let input = send[c] + fb * feedback;
                    out[c] = self.lines[c].process(input, delay_samples);
                    self.feedback[c] = out[c];
                }
                out
            }
            DelayType::PingPong => {
                let mono_in = (send[0] + send[1]) * 0.5;
                let fb_l = ftz(self.feedback[0], 0.0001);
                let fb_r = ftz(self.feedback[1], 0.0001);

                let input_l = mono_in + fb_r * feedback;
                let input_r = fb_l * feedback;

                let out_l = self.lines[0].process(input_l, delay_samples);
                let out_r = self.lines[1].process(input_r, delay_samples);

                self.feedback[0] = out_l;
                self.feedback[1] = out_r;
                [out_l, out_r]
            }
            DelayType::Tape => {
                const DAMP: f32 = 0.35;
                let mut out = [0.0; CHANNELS];
                for c in 0..CHANNELS {
                    let fb_raw = ftz(self.feedback[c], 0.0001);
                    let fb = self.lp[c] + DAMP * (fb_raw - self.lp[c]);
                    self.lp[c] = fb;

                    let input = send[c] + fb * feedback;
                    out[c] = self.lines[c].process(input, delay_samples);
                    self.feedback[c] = out[c];
                }
                out
            }
            DelayType::Multitap => {
                let t = delay_samples;
                let swing = feedback;

                let tap1 = delay_samples;
                let tap2 = (t * (0.5 + swing * 0.167)).max(1.0);
                let tap3 = (t * (0.25 + swing * 0.083)).max(1.0);
                let tap4 = (t * (0.125 + swing * 0.042)).max(1.0);

                let mut out = [0.0; CHANNELS];
                for c in 0..CHANNELS {
                    let fb = ftz(self.feedback[c], 0.0001);
                    let input = send[c] + fb * 0.5;
                    self.lines[c].write(input);

                    let out1 = self.lines[c].read_at(tap1);
                    let out2 = self.lines[c].read_at(tap2) * 0.7;
                    let out3 = self.lines[c].read_at(tap3) * 0.5;
                    let out4 = self.lines[c].read_at(tap4) * 0.35;

                    out[c] = out1 + out2 + out3 + out4;
                    self.feedback[c] = out1;
                }
                out
            }
        }
    }

    /// Block variant: processes `n` stereo frames in-place. Algorithm dispatch
    /// stays inside the per-sample `process` kernel; this wrapper hoists nothing
    /// (call-site hoisting is reserved for later phases).
    #[inline]
    pub fn process_block(&mut self, buf: &mut [StereoFrame], n: usize) {
        for slot in buf.iter_mut().take(n) {
            *slot = self.process(*slot);
        }
    }

    pub fn clear(&mut self) {
        for line in &mut self.lines {
            line.clear();
        }
        self.feedback = [0.0; CHANNELS];
        self.lp = [0.0; CHANNELS];
        self.time_smooth = -1.0;
    }
}
