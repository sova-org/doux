//! Fast approximations for common mathematical functions.
//!
//! This module provides SIMD-friendly, branch-minimal implementations of
//! transcendental functions optimized for audio synthesis. These trade some
//! accuracy for significant performance gains in tight DSP loops.
//!
//! # Accuracy
//!
//! | Function   | Typical Error |
//! |------------|---------------|
//! | `exp2f`    | < 0.01%       |
//! | `log2f`    | < 0.1%        |
//! | `sinf`     | < 0.01%       |
//! | `par_sinf` | < 0.1%        |
//! | `pow10`    | < 0.1%        |
//!
//! # Implementation Notes
//!
//! All functions are division-free (except `fast_tanh` and `atan2f`).
//! The logarithm and exponential functions exploit IEEE 754 float bit layout,
//! extracting and manipulating exponent/mantissa fields directly. Trigonometric
//! functions use minimax polynomial approximations.

use std::f32::consts::{LOG2_10, PI};

/// Bit position of the exponent field in IEEE 754 single precision.
const F32_EXP_SHIFT: i32 = 23;

/// Exponent bias for IEEE 754 single precision.
const F32_BIAS: i32 = 127;

/// Fast base-2 logarithm approximation.
///
/// Extracts the IEEE 754 exponent and applies a degree-3 minimax polynomial
/// on the mantissa. Division-free.
///
/// # Panics
///
/// Does not panic, but returns meaningless results for `x <= 0`.
#[inline]
pub fn log2f(x: f32) -> f32 {
    let bits = x.to_bits();
    let e = ((bits >> F32_EXP_SHIFT as u32) & 0xFF) as f32 - F32_BIAS as f32;
    let m = f32::from_bits((bits & 0x007F_FFFF) | ((F32_BIAS as u32) << F32_EXP_SHIFT as u32));
    e + (-2.153_620_7 + m * (3.047_884_2 + m * (-1.051_875 + m * 0.158_248_71)))
}

/// Fast base-2 exponential approximation.
///
/// Separates the integer and fractional parts of the exponent. The integer
/// part is computed via bit manipulation, the fractional part uses a
/// degree-4 minimax polynomial on [0, 1).
#[inline]
pub fn exp2f(x: f32) -> f32 {
    if x < -126.0 {
        return 0.0;
    }
    if x > 126.0 {
        return f32::INFINITY;
    }
    let xf = x.floor();
    let f = x - xf;
    let exp_bits = ((127 + xf as i32) as u32) << 23;
    let ystep = f32::from_bits(exp_bits);
    ystep * (1.0 + f * (0.693_147_2 + f * (0.240_226_5 + f * (0.055_504_1 + f * 0.009_618_1))))
}

/// Fast power function: `x^y`.
///
/// Computed as `2^(y * log2(x))` using fast approximations.
///
/// # Special Cases
///
/// - Returns `NAN` if `x < 0`
/// - Returns `0.0` if `x == 0`
/// - Returns `1.0` if `y == 0`
#[inline]
pub fn powf(x: f32, y: f32) -> f32 {
    if x < 0.0 {
        return f32::NAN;
    }
    if x == 0.0 {
        return 0.0;
    }
    if y == 0.0 {
        return 1.0;
    }
    exp2f(y * log2f(x))
}

/// Computes `0.5^x` (equivalently `2^(-x)`).
///
/// Useful for exponential decay calculations where the half-life is the
/// natural unit.
#[inline]
pub fn pow1half(x: f32) -> f32 {
    exp2f(-x)
}

/// Fast `10^x` approximation.
///
/// Useful for decibel conversions: `pow10(db / 20.0)` gives amplitude ratio.
#[inline]
pub fn pow10(x: f32) -> f32 {
    exp2f(x * LOG2_10)
}

/// Wraps angle to the range `[-π, π]`.
///
/// Essential for maintaining phase coherence in oscillators over long
/// running times, preventing floating-point precision loss.
#[inline]
pub fn modpi(x: f32) -> f32 {
    let mut x = x + PI;
    x *= 0.5 / PI;
    x -= x.floor();
    x *= 2.0 * PI;
    x - PI
}

/// Parabolic sine approximation with weight correction.
///
/// Very fast with good accuracy. Uses a parabola fitted at 0, ±π/2, and ±π,
/// then applies the Coranac weight correction to reduce max error from
/// ~12% to ~0.06%.
#[inline]
pub fn par_sinf(x: f32) -> f32 {
    let x = modpi(x);
    let y = 0.405_284_73 * x * (PI - x.abs());
    y * (0.775 + 0.225 * y.abs())
}

/// Parabolic cosine approximation.
///
/// Phase-shifted [`par_sinf`].
#[inline]
pub fn par_cosf(x: f32) -> f32 {
    par_sinf(x + 0.5 * PI)
}

/// Fast sine approximation using degree-5 minimax polynomial.
///
/// Higher accuracy than [`par_sinf`] and significantly faster than
/// `std::f32::sin`. Division-free.
#[inline]
pub fn sinf(x: f32) -> f32 {
    let x = 4.0 * (x * (0.5 / PI) - (x * (0.5 / PI) + 0.75).floor() + 0.25).abs() - 1.0;
    let x = x * (PI / 2.0);
    let x2 = x * x;
    x * (0.999_979_38 + x2 * (-0.166_498_13 + x2 * 0.007_997_90))
}

/// Fast cosine approximation.
///
/// Phase-shifted [`sinf`].
#[inline]
pub fn cosf(x: f32) -> f32 {
    sinf(x + 0.5 * PI)
}

/// Flush to zero: clamps small values to zero.
///
/// Prevents denormalized floating-point numbers which can cause severe
/// performance degradation in audio processing loops on some architectures.
#[inline]
pub fn ftz(x: f32, limit: f32) -> f32 {
    if x < limit && x > -limit {
        0.0
    } else {
        x
    }
}

/// Fast hyperbolic tangent approximation (f64).
///
/// Rational cubic preserving odd symmetry. Replaces expensive `f64::tanh()`
/// in the ladder filter (~5 cycles vs ~20-50).
#[inline]
pub fn fast_tanh(x: f64) -> f64 {
    let x = x.clamp(-3.0, 3.0);
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

/// Fast hyperbolic tangent approximation (f32).
///
/// Same rational cubic as [`fast_tanh`] but in single precision.
#[inline]
pub fn fast_tanh_f32(x: f32) -> f32 {
    let x = x.clamp(-3.0, 3.0);
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

/// Fast atan2 approximation.
///
/// Uses octant reduction with a linear-corrected polynomial:
/// `atan(a) ≈ a * (π/4 + 0.273·(1-a))` for `0 ≤ a ≤ 1`.
/// Max error < 0.005 rad.
#[inline]
pub fn atan2f(y: f32, x: f32) -> f32 {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

    let ax = x.abs();
    let ay = y.abs();
    let (min, max) = if ax < ay { (ax, ay) } else { (ay, ax) };

    if max == 0.0 {
        return 0.0;
    }

    let a = min / max;
    let r = a * (FRAC_PI_4 + 0.273 * (1.0 - a));

    let r = if ax < ay { FRAC_PI_2 - r } else { r };
    let r = if x < 0.0 { PI - r } else { r };
    if y < 0.0 { -r } else { r }
}

/// Fast tangent approximation via `sinf(x) / cosf(x)`.
#[inline]
pub fn fast_tan(x: f32) -> f32 {
    sinf(x) / cosf(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exp2f() {
        for i in -10..10 {
            let x = i as f32 * 0.5;
            let fast = exp2f(x);
            let std = 2.0_f32.powf(x);
            assert!(
                (fast - std).abs() < 0.001,
                "exp2f({x}) = {fast} vs std {std}"
            );
        }
    }

    #[test]
    fn test_exp2f_extreme_inputs() {
        assert_eq!(exp2f(-200.0), 0.0);
        assert_eq!(exp2f(-127.0), 0.0);
        assert!(exp2f(-126.0).is_finite());
        assert!(exp2f(126.0).is_finite());
        assert_eq!(exp2f(127.0), f32::INFINITY);
    }

    #[test]
    fn test_sinf() {
        for i in 0..20 {
            let x = (i as f32 - 10.0) * 0.5;
            let fast = sinf(x);
            let std = x.sin();
            assert!((fast - std).abs() < 0.01, "sinf({x}) = {fast} vs std {std}");
        }
    }

    #[test]
    fn test_pow10() {
        for i in -5..5 {
            let x = i as f32 * 0.5;
            let fast = pow10(x);
            let std = 10.0_f32.powf(x);
            assert!(
                (fast - std).abs() / std < 0.01,
                "pow10({x}) = {fast} vs std {std}"
            );
        }
    }

    #[test]
    fn test_fast_tanh() {
        for i in -30..=30 {
            let x = i as f64 * 0.1;
            let fast = fast_tanh(x);
            let std = x.tanh();
            let err = (fast - std).abs();
            assert!(err < 0.03, "fast_tanh({x}) = {fast} vs std {std}, err={err}");
        }
    }
}
