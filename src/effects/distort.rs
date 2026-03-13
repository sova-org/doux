//! Waveshaping distortion effects.
//!
//! Three flavors of nonlinear distortion:
//! - [`distort`]: Soft saturation (tube-like warmth)
//! - `fold`: Wavefolding (complex harmonics)
//! - [`wrap`]: Phase wrapping (harsh, digital)

use crate::dsp::{exp2f, expm1f, sinf};
use crate::types::{ModuleInfo, ModuleGroup, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "distort",
    description: "Waveshaping distortion (saturation, wavefolding, phase wrapping)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo { name: "distort", aliases: &[], description: "soft saturation amount", default: "0.0", min: 0.0, max: 10.0 },
        ParamInfo { name: "fold", aliases: &[], description: "sine wavefolding amount", default: "0.0", min: 0.0, max: 1.0 },
        ParamInfo { name: "wrap", aliases: &[], description: "phase wrapping amount", default: "0.0", min: 0.0, max: 10.0 },
        ParamInfo { name: "distortvol", aliases: &[], description: "output volume compensation", default: "1.0", min: 0.0, max: 2.0 },
    ],
};

/// Soft-knee saturation with adjustable drive.
///
/// Uses `x / (1 + k|x|)` transfer function for smooth clipping.
/// Higher `amount` = more compression and harmonics.
#[inline]
pub fn distort(input: f32, amount: f32, postgain: f32) -> f32 {
    let k = expm1f(amount);
    ((1.0 + k) * input / (1.0 + k * input.abs())) * postgain
}

/// Sine wavefolder with normalized amount in [0.0, 1.0].
///
/// Maps amount exponentially to internal gain [1, 16] via `exp2f(amount * 4)`,
/// then applies `sin(x × gain × π/2)`.
#[inline]
pub fn fold(input: f32, amount: f32) -> f32 {
    let gain = exp2f(amount * 4.0);
    sinf(input * gain * std::f32::consts::FRAC_PI_2)
}

/// Wraps signal into `[-1, 1]` range using modulo.
///
/// Creates harsh, digital-sounding distortion with discontinuities.
/// `wraps` controls how many times the signal can wrap.
#[inline]
pub fn wrap(input: f32, wraps: f32) -> f32 {
    if wraps < 1.0 {
        return input;
    }
    let x = input * (1.0 + wraps);
    (x + 1.0).rem_euclid(2.0) - 1.0
}
