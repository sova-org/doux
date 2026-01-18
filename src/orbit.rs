use crate::effects::Comb;
use crate::fastmath::ftz;
use crate::types::{DelayType, CHANNELS};

const MAX_DELAY_SAMPLES: usize = 48000;
const SILENCE_THRESHOLD: f32 = 1e-7;
const SILENCE_HOLDOFF: u32 = 48000;

#[derive(Clone, Copy, Default)]
pub struct EffectParams {
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_type: DelayType,
    pub verb_decay: f32,
    pub verb_damp: f32,
    pub verb_predelay: f32,
    pub verb_diff: f32,
    pub comb_freq: f32,
    pub comb_feedback: f32,
    pub comb_damp: f32,
}

#[derive(Clone)]
pub struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayLine {
    pub fn new(max_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_samples],
            write_pos: 0,
        }
    }

    pub fn process(&mut self, input: f32, delay_samples: usize) -> f32 {
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

    pub fn read_at(&self, delay_samples: usize) -> f32 {
        let delay_samples = delay_samples.min(self.buffer.len() - 1);
        let read_pos = if self.write_pos >= delay_samples {
            self.write_pos - delay_samples
        } else {
            self.buffer.len() - (delay_samples - self.write_pos)
        };
        self.buffer[read_pos]
    }

    pub fn write(&mut self, input: f32) {
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
    }
}

impl Default for DelayLine {
    fn default() -> Self {
        Self::new(MAX_DELAY_SAMPLES)
    }
}

const REVERB_SR_REF: f32 = 29761.0;

fn scale_delay(samples: usize, sr: f32) -> usize {
    ((samples as f32 * sr / REVERB_SR_REF) as usize).max(1)
}

#[derive(Clone)]
pub struct ReverbBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
}

impl ReverbBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            write_pos: 0,
        }
    }

    pub fn write(&mut self, value: f32) {
        self.buffer[self.write_pos] = value;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }

    pub fn read(&self, delay: usize) -> f32 {
        let delay = delay.min(self.buffer.len() - 1);
        let pos = if self.write_pos >= delay {
            self.write_pos - delay
        } else {
            self.buffer.len() - (delay - self.write_pos)
        };
        self.buffer[pos]
    }

    pub fn read_write(&mut self, value: f32, delay: usize) -> f32 {
        let out = self.read(delay);
        self.write(value);
        out
    }

    pub fn allpass(&mut self, input: f32, delay: usize, coeff: f32) -> f32 {
        let delayed = self.read(delay);
        let v = input - coeff * delayed;
        self.write(v);
        delayed + coeff * v
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
    }
}

#[derive(Clone)]
pub struct DattorroVerb {
    pre_delay: ReverbBuffer,
    in_diff1: ReverbBuffer,
    in_diff2: ReverbBuffer,
    in_diff3: ReverbBuffer,
    in_diff4: ReverbBuffer,
    decay_diff1_l: ReverbBuffer,
    delay1_l: ReverbBuffer,
    decay_diff2_l: ReverbBuffer,
    delay2_l: ReverbBuffer,
    decay_diff1_r: ReverbBuffer,
    delay1_r: ReverbBuffer,
    decay_diff2_r: ReverbBuffer,
    delay2_r: ReverbBuffer,
    damp_l: f32,
    damp_r: f32,
    pre_delay_len: usize,
    in_diff1_len: usize,
    in_diff2_len: usize,
    in_diff3_len: usize,
    in_diff4_len: usize,
    decay_diff1_l_len: usize,
    delay1_l_len: usize,
    decay_diff2_l_len: usize,
    delay2_l_len: usize,
    decay_diff1_r_len: usize,
    delay1_r_len: usize,
    decay_diff2_r_len: usize,
    delay2_r_len: usize,
    tap1_l: usize,
    tap2_l: usize,
    tap3_l: usize,
    tap4_l: usize,
    tap5_l: usize,
    tap6_l: usize,
    tap7_l: usize,
    tap1_r: usize,
    tap2_r: usize,
    tap3_r: usize,
    tap4_r: usize,
    tap5_r: usize,
    tap6_r: usize,
    tap7_r: usize,
}

impl DattorroVerb {
    pub fn new(sr: f32) -> Self {
        let pre_delay_len = scale_delay(4800, sr);
        let in_diff1_len = scale_delay(142, sr);
        let in_diff2_len = scale_delay(107, sr);
        let in_diff3_len = scale_delay(379, sr);
        let in_diff4_len = scale_delay(277, sr);
        let decay_diff1_l_len = scale_delay(672, sr);
        let delay1_l_len = scale_delay(4453, sr);
        let decay_diff2_l_len = scale_delay(1800, sr);
        let delay2_l_len = scale_delay(3720, sr);
        let decay_diff1_r_len = scale_delay(908, sr);
        let delay1_r_len = scale_delay(4217, sr);
        let decay_diff2_r_len = scale_delay(2656, sr);
        let delay2_r_len = scale_delay(3163, sr);

        Self {
            pre_delay: ReverbBuffer::new(pre_delay_len + 1),
            in_diff1: ReverbBuffer::new(in_diff1_len + 1),
            in_diff2: ReverbBuffer::new(in_diff2_len + 1),
            in_diff3: ReverbBuffer::new(in_diff3_len + 1),
            in_diff4: ReverbBuffer::new(in_diff4_len + 1),
            decay_diff1_l: ReverbBuffer::new(decay_diff1_l_len + 1),
            delay1_l: ReverbBuffer::new(delay1_l_len + 1),
            decay_diff2_l: ReverbBuffer::new(decay_diff2_l_len + 1),
            delay2_l: ReverbBuffer::new(delay2_l_len + 1),
            decay_diff1_r: ReverbBuffer::new(decay_diff1_r_len + 1),
            delay1_r: ReverbBuffer::new(delay1_r_len + 1),
            decay_diff2_r: ReverbBuffer::new(decay_diff2_r_len + 1),
            delay2_r: ReverbBuffer::new(delay2_r_len + 1),
            damp_l: 0.0,
            damp_r: 0.0,
            pre_delay_len,
            in_diff1_len,
            in_diff2_len,
            in_diff3_len,
            in_diff4_len,
            decay_diff1_l_len,
            delay1_l_len,
            decay_diff2_l_len,
            delay2_l_len,
            decay_diff1_r_len,
            delay1_r_len,
            decay_diff2_r_len,
            delay2_r_len,
            tap1_l: scale_delay(266, sr),
            tap2_l: scale_delay(2974, sr),
            tap3_l: scale_delay(1913, sr),
            tap4_l: scale_delay(1996, sr),
            tap5_l: scale_delay(1990, sr),
            tap6_l: scale_delay(187, sr),
            tap7_l: scale_delay(1066, sr),
            tap1_r: scale_delay(353, sr),
            tap2_r: scale_delay(3627, sr),
            tap3_r: scale_delay(1228, sr),
            tap4_r: scale_delay(2673, sr),
            tap5_r: scale_delay(2111, sr),
            tap6_r: scale_delay(335, sr),
            tap7_r: scale_delay(121, sr),
        }
    }

    pub fn process(
        &mut self,
        input: f32,
        decay: f32,
        damping: f32,
        predelay: f32,
        diffusion: f32,
    ) -> [f32; 2] {
        let decay = decay.clamp(0.0, 0.99);
        let damping = damping.clamp(0.0, 1.0);
        let diffusion = diffusion.clamp(0.0, 1.0);
        let diff1 = 0.75 * diffusion;
        let diff2 = 0.625 * diffusion;
        let decay_diff1 = 0.7 * diffusion;
        let decay_diff2 = 0.5 * diffusion;

        let pre_delay_samples =
            ((predelay * self.pre_delay_len as f32) as usize).min(self.pre_delay_len);
        let input = ftz(input, 0.0001);
        let pre = self.pre_delay.read_write(input, pre_delay_samples);

        let mut x = pre;
        x = self.in_diff1.allpass(x, self.in_diff1_len, diff1);
        x = self.in_diff2.allpass(x, self.in_diff2_len, diff1);
        x = self.in_diff3.allpass(x, self.in_diff3_len, diff2);
        x = self.in_diff4.allpass(x, self.in_diff4_len, diff2);

        let tank_l_in = x + self.delay2_r.read(self.delay2_r_len) * decay;
        let tank_r_in = x + self.delay2_l.read(self.delay2_l_len) * decay;

        let mut l = self
            .decay_diff1_l
            .allpass(tank_l_in, self.decay_diff1_l_len, -decay_diff1);
        l = self.delay1_l.read_write(l, self.delay1_l_len);
        self.damp_l = ftz(l * (1.0 - damping) + self.damp_l * damping, 0.0001);
        l = self.damp_l * decay;
        l = self
            .decay_diff2_l
            .allpass(l, self.decay_diff2_l_len, decay_diff2);
        self.delay2_l.write(l);

        let mut r = self
            .decay_diff1_r
            .allpass(tank_r_in, self.decay_diff1_r_len, -decay_diff1);
        r = self.delay1_r.read_write(r, self.delay1_r_len);
        self.damp_r = ftz(r * (1.0 - damping) + self.damp_r * damping, 0.0001);
        r = self.damp_r * decay;
        r = self
            .decay_diff2_r
            .allpass(r, self.decay_diff2_r_len, decay_diff2);
        self.delay2_r.write(r);

        let out_l = self.delay1_l.read(self.tap1_l) + self.delay1_l.read(self.tap2_l)
            - self.decay_diff2_l.read(self.tap3_l)
            + self.delay2_l.read(self.tap4_l)
            - self.delay1_r.read(self.tap5_r)
            - self.decay_diff2_r.read(self.tap6_r)
            - self.delay2_r.read(self.tap7_r);

        let out_r = self.delay1_r.read(self.tap1_r) + self.delay1_r.read(self.tap2_r)
            - self.decay_diff2_r.read(self.tap3_r)
            + self.delay2_r.read(self.tap4_r)
            - self.delay1_l.read(self.tap5_l)
            - self.decay_diff2_l.read(self.tap6_l)
            - self.delay2_l.read(self.tap7_l);

        [out_l * 0.6, out_r * 0.6]
    }

    pub fn clear(&mut self) {
        self.pre_delay.clear();
        self.in_diff1.clear();
        self.in_diff2.clear();
        self.in_diff3.clear();
        self.in_diff4.clear();
        self.decay_diff1_l.clear();
        self.delay1_l.clear();
        self.decay_diff2_l.clear();
        self.delay2_l.clear();
        self.decay_diff1_r.clear();
        self.delay1_r.clear();
        self.decay_diff2_r.clear();
        self.delay2_r.clear();
        self.damp_l = 0.0;
        self.damp_r = 0.0;
    }
}

pub struct Orbit {
    pub delay: [DelayLine; CHANNELS],
    pub delay_send: [f32; CHANNELS],
    pub delay_out: [f32; CHANNELS],
    pub delay_feedback: [f32; CHANNELS],
    pub delay_lp: [f32; CHANNELS],
    pub verb: DattorroVerb,
    pub verb_send: [f32; CHANNELS],
    pub verb_out: [f32; CHANNELS],
    pub comb: Comb,
    pub comb_send: [f32; CHANNELS],
    pub comb_out: [f32; CHANNELS],
    pub sr: f32,
    silent_samples: u32,
}

impl Orbit {
    pub fn new(sr: f32) -> Self {
        Self {
            delay: [DelayLine::default(), DelayLine::default()],
            delay_send: [0.0; CHANNELS],
            delay_out: [0.0; CHANNELS],
            delay_feedback: [0.0; CHANNELS],
            delay_lp: [0.0; CHANNELS],
            verb: DattorroVerb::new(sr),
            verb_send: [0.0; CHANNELS],
            verb_out: [0.0; CHANNELS],
            comb: Comb::default(),
            comb_send: [0.0; CHANNELS],
            comb_out: [0.0; CHANNELS],
            sr,
            silent_samples: SILENCE_HOLDOFF + 1,
        }
    }

    pub fn clear_sends(&mut self) {
        self.delay_send = [0.0; CHANNELS];
        self.verb_send = [0.0; CHANNELS];
        self.comb_send = [0.0; CHANNELS];
    }

    pub fn add_delay_send(&mut self, ch: usize, value: f32) {
        self.delay_send[ch] += value;
    }

    pub fn add_verb_send(&mut self, ch: usize, value: f32) {
        self.verb_send[ch] += value;
    }

    pub fn add_comb_send(&mut self, ch: usize, value: f32) {
        self.comb_send[ch] += value;
    }

    pub fn process(&mut self, p: &EffectParams) {
        let has_input = self.delay_send[0] != 0.0
            || self.delay_send[1] != 0.0
            || self.verb_send[0] != 0.0
            || self.verb_send[1] != 0.0
            || self.comb_send[0] != 0.0
            || self.comb_send[1] != 0.0;

        if has_input {
            self.silent_samples = 0;
        } else if self.silent_samples > SILENCE_HOLDOFF {
            self.delay_out = [0.0; CHANNELS];
            self.verb_out = [0.0; CHANNELS];
            self.comb_out = [0.0; CHANNELS];
            return;
        }

        let delay_samples = ((p.delay_time * self.sr) as usize).min(MAX_DELAY_SAMPLES - 1);
        let feedback = p.delay_feedback.clamp(0.0, 0.95);

        match p.delay_type {
            DelayType::Standard => {
                for c in 0..CHANNELS {
                    let fb = ftz(self.delay_feedback[c], 0.0001);
                    let input = self.delay_send[c] + fb * feedback;
                    self.delay_out[c] = self.delay[c].process(input, delay_samples);
                    self.delay_feedback[c] = self.delay_out[c];
                }
            }
            DelayType::PingPong => {
                // True ping-pong: mono input → L only, then bounces L↔R
                let mono_in = (self.delay_send[0] + self.delay_send[1]) * 0.5;
                let fb_l = ftz(self.delay_feedback[0], 0.0001);
                let fb_r = ftz(self.delay_feedback[1], 0.0001);

                // L gets new input + feedback from R
                let input_l = mono_in + fb_r * feedback;
                // R gets ONLY feedback from L (no direct input - this creates the bounce)
                let input_r = fb_l * feedback;

                self.delay_out[0] = self.delay[0].process(input_l, delay_samples);
                self.delay_out[1] = self.delay[1].process(input_r, delay_samples);

                self.delay_feedback[0] = self.delay_out[0];
                self.delay_feedback[1] = self.delay_out[1];
            }
            DelayType::Tape => {
                // Tape delay: one-pole lowpass in feedback (darkening each repeat)
                const DAMP: f32 = 0.35;
                for c in 0..CHANNELS {
                    // Apply lowpass to feedback before using it
                    let fb_raw = ftz(self.delay_feedback[c], 0.0001);
                    let fb = self.delay_lp[c] + DAMP * (fb_raw - self.delay_lp[c]);
                    self.delay_lp[c] = fb;

                    let input = self.delay_send[c] + fb * feedback;
                    self.delay_out[c] = self.delay[c].process(input, delay_samples);
                    self.delay_feedback[c] = self.delay_out[c];
                }
            }
            DelayType::Multitap => {
                // Multitap: 4 taps with straight-to-triplet morph
                // feedback 0 = straight (1, 1/2, 1/4, 1/8)
                // feedback 1 = triplet (1, 2/3, 1/3, 1/6)
                // in between = swing
                let swing = feedback;
                let t = delay_samples as f32;

                let tap1 = delay_samples;
                let tap2 = (t * (0.5 + swing * 0.167)).max(1.0) as usize; // 1/2 → 2/3
                let tap3 = (t * (0.25 + swing * 0.083)).max(1.0) as usize; // 1/4 → 1/3
                let tap4 = (t * (0.125 + swing * 0.042)).max(1.0) as usize; // 1/8 → 1/6

                for c in 0..CHANNELS {
                    let fb = ftz(self.delay_feedback[c], 0.0001);
                    let input = self.delay_send[c] + fb * 0.5;
                    self.delay[c].write(input);

                    let out1 = self.delay[c].read_at(tap1);
                    let out2 = self.delay[c].read_at(tap2) * 0.7;
                    let out3 = self.delay[c].read_at(tap3) * 0.5;
                    let out4 = self.delay[c].read_at(tap4) * 0.35;

                    self.delay_out[c] = out1 + out2 + out3 + out4;
                    self.delay_feedback[c] = out1;
                }
            }
        }

        let verb_input = (self.verb_send[0] + self.verb_send[1]) * 0.5;
        let verb_stereo = self.verb.process(
            verb_input,
            p.verb_decay,
            p.verb_damp,
            p.verb_predelay,
            p.verb_diff,
        );
        self.verb_out[0] = verb_stereo[0];
        self.verb_out[1] = verb_stereo[1];

        // Comb filter (mono in, mono out to both channels)
        let comb_input = (self.comb_send[0] + self.comb_send[1]) * 0.5;
        let comb_out = self.comb.process(
            comb_input,
            p.comb_freq,
            p.comb_feedback,
            p.comb_damp,
            self.sr,
        );
        self.comb_out[0] = comb_out;
        self.comb_out[1] = comb_out;

        let energy = self.delay_out[0].abs()
            + self.delay_out[1].abs()
            + self.verb_out[0].abs()
            + self.verb_out[1].abs()
            + self.comb_out[0].abs()
            + self.comb_out[1].abs();

        if energy < SILENCE_THRESHOLD {
            self.silent_samples = self.silent_samples.saturating_add(1);
        } else {
            self.silent_samples = 0;
        }
    }
}
