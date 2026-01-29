use crate::dsp::ftz;
use crate::types::{DelayType, CHANNELS};

const MAX_DELAY_SAMPLES: usize = 48000;

#[derive(Clone)]
struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayLine {
    fn new(max_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_samples],
            write_pos: 0,
        }
    }

    fn process(&mut self, input: f32, delay_samples: usize) -> f32 {
        let delay_samples = delay_samples.min(self.buffer.len() - 1);
        self.buffer[self.write_pos] = input;

        let read_pos = if self.write_pos >= delay_samples {
            self.write_pos - delay_samples
        } else {
            self.buffer.len() - (delay_samples - self.write_pos)
        };

        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        self.buffer[read_pos]
    }

    fn read_at(&self, delay_samples: usize) -> f32 {
        let delay_samples = delay_samples.min(self.buffer.len() - 1);
        let read_pos = if self.write_pos >= delay_samples {
            self.write_pos - delay_samples
        } else {
            self.buffer.len() - (delay_samples - self.write_pos)
        };
        self.buffer[read_pos]
    }

    fn write(&mut self, input: f32) {
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
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
    pub sr: f32,
}

#[derive(Clone, Default)]
pub struct Delay {
    lines: [DelayLine; CHANNELS],
    feedback: [f32; CHANNELS],
    lp: [f32; CHANNELS],
}

impl Delay {
    pub fn new() -> Self {
        Self {
            lines: [DelayLine::default(), DelayLine::default()],
            feedback: [0.0; CHANNELS],
            lp: [0.0; CHANNELS],
        }
    }

    pub fn process(&mut self, send: [f32; CHANNELS], p: &DelayParams) -> [f32; CHANNELS] {
        let delay_samples = ((p.time * p.sr) as usize).min(MAX_DELAY_SAMPLES - 1);
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
                let t = delay_samples as f32;
                let swing = feedback;

                let tap1 = delay_samples;
                let tap2 = (t * (0.5 + swing * 0.167)).max(1.0) as usize;
                let tap3 = (t * (0.25 + swing * 0.083)).max(1.0) as usize;
                let tap4 = (t * (0.125 + swing * 0.042)).max(1.0) as usize;

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

    pub fn clear(&mut self) {
        for line in &mut self.lines {
            line.clear();
        }
        self.feedback = [0.0; CHANNELS];
        self.lp = [0.0; CHANNELS];
    }
}
