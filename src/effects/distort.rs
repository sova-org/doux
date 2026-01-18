//! Waveshaping distortion effects.
//!
//! Three flavors of nonlinear distortion:
//! - [`distort`]: Soft saturation (tube-like warmth)
//! - [`fold`]: Wavefolding (complex harmonics)
//! - [`wrap`]: Phase wrapping (harsh, digital)

use crate::fastmath::{expm1f, sinf};

/// Soft-knee saturation with adjustable drive.
///
/// Uses `x / (1 + k|x|)` transfer function for smooth clipping.
/// Higher `amount` = more compression and harmonics.
pub fn distort(input: f32, amount: f32, postgain: f32) -> f32 {
    let k = expm1f(amount);
    ((1.0 + k) * input / (1.0 + k * input.abs())) * postgain
}

/// Sine wavefolder.
///
/// Folds the waveform back on itself using `sin(x × amount × π/2)`.
/// Creates rich harmonic content without hard clipping.
pub fn fold(input: f32, amount: f32) -> f32 {
    sinf(input * amount * std::f32::consts::FRAC_PI_2)
}

/// Wraps signal into `[-1, 1]` range using modulo.
///
/// Creates harsh, digital-sounding distortion with discontinuities.
/// `wraps` controls how many times the signal can wrap.
pub fn wrap(input: f32, wraps: f32) -> f32 {
    if wraps < 1.0 {
        return input;
    }
    let x = input * (1.0 + wraps);
    (x + 1.0).rem_euclid(2.0) - 1.0
}
