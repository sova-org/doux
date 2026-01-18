//! Sample rate reduction (bitcrusher-style decimation).
//!
//! Reduces the effective sample rate by holding each sample value for multiple
//! output samples, creating the characteristic "crunchy" lo-fi sound of early
//! samplers and video game consoles.
//!
//! # Example
//!
//! With `factor = 4` at 48kHz, the effective sample rate becomes 12kHz:
//!
//! ```text
//! Input:  [a, b, c, d, e, f, g, h, ...]
//! Output: [a, a, a, a, e, e, e, e, ...]
//! ```

/// Sample-and-hold decimator for lo-fi effects.
///
/// Holds input values for `factor` samples, reducing effective sample rate.
/// Often combined with bit depth reduction for full bitcrusher effects.
#[derive(Clone, Copy, Default)]
pub struct Coarse {
    /// Currently held sample value.
    hold: f32,
    /// Sample counter (0 to factor-1).
    t: usize,
}

impl Coarse {
    /// Processes one sample through the decimator.
    ///
    /// # Parameters
    ///
    /// - `input`: Input sample
    /// - `factor`: Decimation factor (1.0 = bypass, 2.0 = half rate, etc.)
    ///
    /// # Returns
    ///
    /// The held sample value. Updates only when the internal counter wraps.
    pub fn process(&mut self, input: f32, factor: f32) -> f32 {
        let n = factor.max(1.0) as usize;
        if self.t == 0 {
            self.hold = input;
        }
        self.t = (self.t + 1) % n;
        self.hold
    }
}
