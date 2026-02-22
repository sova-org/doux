//! Bit depth reduction for lo-fi effects.
//!
//! Quantizes amplitude to fewer bits, creating the stepped distortion
//! characteristic of early digital audio. Pair with [`super::coarse`] for
//! full bitcrusher (sample rate + bit depth reduction).

use crate::dsp::exp2f;

/// Reduces bit depth by quantizing to `2^(bits-1)` levels.
///
/// - `bits = 16`: Near-transparent (CD quality)
/// - `bits = 8`: Classic 8-bit crunch
/// - `bits = 4`: Heavily degraded
/// - `bits = 1`: Square wave (extreme)
#[inline]
pub fn crush(input: f32, bits: f32) -> f32 {
    let bits = bits.max(1.0);
    let x = exp2f(bits - 1.0);
    let inv_x = 1.0 / x;
    (input * x).round() * inv_x
}
