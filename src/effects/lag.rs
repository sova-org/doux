//! One-pole smoothing filter (slew limiter).
//!
//! Smooths abrupt parameter changes to prevent clicks and zipper noise.
//! Higher rate = slower response.

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
}
