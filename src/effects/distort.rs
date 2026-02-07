//! Waveshaping distortion effects.
//!
//! Three flavors of nonlinear distortion:
//! - [`distort`]: Soft saturation (tube-like warmth)
//! - `fold`: Wavefolding (complex harmonics)
//! - [`wrap`]: Phase wrapping (harsh, digital)

use crate::dsp::{exp2f, expm1f, sinf};

/// Soft-knee saturation with adjustable drive.
///
/// Uses `x / (1 + k|x|)` transfer function for smooth clipping.
/// Higher `amount` = more compression and harmonics.
pub fn distort(input: f32, amount: f32, postgain: f32) -> f32 {
    let k = expm1f(amount);
    ((1.0 + k) * input / (1.0 + k * input.abs())) * postgain
}

/// Sine wavefolder with normalized amount in [0.0, 1.0].
///
/// Maps amount exponentially to internal gain [1, 16] via `exp2f(amount * 4)`,
/// then applies `sin(x × gain × π/2)`.
pub fn fold(input: f32, amount: f32) -> f32 {
    let gain = exp2f(amount * 4.0);
    sinf(input * gain * std::f32::consts::FRAC_PI_2)
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
