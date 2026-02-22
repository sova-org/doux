use crate::dsp::fast_tan;
use std::f32::consts::PI;

const NUM_STAGES: usize = 12;

#[derive(Clone, Copy)]
struct Allpass1 {
    x1: f32,
    y1: f32,
}

impl Default for Allpass1 {
    fn default() -> Self {
        Self { x1: 0.0, y1: 0.0 }
    }
}

impl Allpass1 {
    #[inline]
    fn process(&mut self, input: f32, a: f32) -> f32 {
        let y = a * input + self.x1 - a * self.y1;
        self.x1 = input;
        self.y1 = y;
        y
    }
}

#[derive(Clone, Copy)]
pub struct Smear {
    stages: [Allpass1; NUM_STAGES],
    prev_out: f32,
    cached_freq: f32,
    cached_sr: f32,
    cached_a: f32,
}

impl Default for Smear {
    fn default() -> Self {
        Self {
            stages: [Allpass1::default(); NUM_STAGES],
            prev_out: 0.0,
            cached_freq: 0.0,
            cached_sr: 0.0,
            cached_a: 0.0,
        }
    }
}

impl Smear {
    #[inline]
    pub fn process(&mut self, input: f32, mix: f32, freq: f32, feedback: f32, sr: f32) -> f32 {
        if freq != self.cached_freq || sr != self.cached_sr {
            self.cached_freq = freq;
            self.cached_sr = sr;
            let t = (PI * freq / sr).min(PI * 0.4999);
            let tan_t = fast_tan(t);
            self.cached_a = (tan_t - 1.0) / (tan_t + 1.0);
        }

        let x = input + self.prev_out * feedback;
        let mut wet = x;
        for stage in &mut self.stages {
            wet = stage.process(wet, self.cached_a);
        }
        self.prev_out = wet;

        input * (1.0 - mix) + wet * mix
    }
}
