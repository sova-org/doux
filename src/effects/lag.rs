//! One-pole smoothing filter (slew limiter).
//!
//! Smooths abrupt parameter changes to prevent clicks and zipper noise.
//! Higher rate = slower response.

use crate::types::StereoFrame;

/// One-pole lowpass for parameter smoothing.
#[derive(Clone, Copy, Default)]
pub struct Lag {
    /// Current smoothed value.
    pub s: f32,
}

impl Lag {
    /// Moves toward `input` at a rate controlled by `rate × lag_unit`.
    ///
    /// - `rate`: Smoothing factor (higher = slower)
    /// - `lag_unit`: Scaling factor (typically sample-rate dependent)
    #[inline]
    pub fn update(&mut self, input: f32, rate: f32, lag_unit: f32) -> f32 {
        let coeff = 1.0 / (rate * lag_unit).max(1.0);
        self.s += coeff * (input - self.s);
        self.s
    }

    /// Block variant: smooths channel `ch` of `buf[..n]` in place. `coeff`
    /// hoists to block entry.
    #[inline]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        rate: f32,
        lag_unit: f32,
    ) {
        let coeff = 1.0 / (rate * lag_unit).max(1.0);
        for slot in buf.iter_mut().take(n) {
            self.s += coeff * (slot[ch] - self.s);
            slot[ch] = self.s;
        }
    }
}
