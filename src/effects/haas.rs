//! Haas effect — short delay on one channel for spatial placement.

use crate::dsp::{ms_to_samples, DelayLine};
use crate::types::StereoFrame;

const BUFFER_SIZE: usize = 2048;

/// Mono delay line for Haas effect.
#[derive(Clone, Copy, Default)]
pub struct Haas {
    delay: DelayLine<BUFFER_SIZE>,
}

impl Haas {
    /// Delays the input by `ms` milliseconds. Returns the delayed sample.
    #[inline]
    pub fn process(&mut self, input: f32, ms: f32, sr: f32) -> f32 {
        self.delay.write(input);
        let delay_samples = ms_to_samples(ms, sr).clamp(1.0, (BUFFER_SIZE - 2) as f32);
        self.delay.read(delay_samples)
    }

    /// Block variant: delays the right channel of each frame in-place.
    #[inline]
    pub fn process_block(&mut self, buf: &mut [StereoFrame], n: usize, ms: f32, sr: f32) {
        for frame in buf.iter_mut().take(n) {
            frame[1] = self.process(frame[1], ms, sr);
        }
    }
}
