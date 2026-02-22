//! Fast approximations for common mathematical functions.
//!
//! This module provides SIMD-friendly, branch-minimal implementations of
//! transcendental functions optimized for audio synthesis. These trade some
//! accuracy for significant performance gains in tight DSP loops.
//!
//! # Accuracy
//!
//! | Function | Typical Error |
//! |----------|---------------|
//! | `exp2f`  | < 0.1%        |
//! | `log2f`  | < 0.1%        |
//! | `sinf`   | < 1%          |
//! | `pow10`  | < 1%          |
//!
//! # Implementation Notes
//!
//! The logarithm and exponential functions exploit IEEE 754 float bit layout,
//! extracting and manipulating exponent/mantissa fields directly. Trigonometric
//! functions use rational polynomial approximations.

use std::f32::consts::{LOG2_10, LOG2_E, PI, SQRT_2};

/// Bit position of the exponent field in IEEE 754 single precision.
const F32_EXP_SHIFT: i32 = 23;

/// Exponent bias for IEEE 754 single precision.
const F32_BIAS: i32 = 127;

/// Fast base-2 logarithm approximation.
///
/// Uses IEEE 754 bit manipulation to extract the exponent, then applies a
/// rational polynomial correction for the mantissa contribution.
///
/// # Panics
///
/// Does not panic, but returns meaningless results for `x <= 0`.
#[inline]
pub fn log2f(x: f32) -> f32 {
    let bits = x.to_bits();
    let mantissa_bits = bits & ((1 << F32_EXP_SHIFT) - 1);
    let biased_mantissa = f32::from_bits(mantissa_bits | ((F32_BIAS as u32 - 1) << F32_EXP_SHIFT));

    let y = bits as f32 * (1.0 / (1 << F32_EXP_SHIFT) as f32);
    y - 124.225_45 - 1.498_030_3 * biased_mantissa - 1.725_88 / (0.352_088_72 + biased_mantissa)
}

/// Fast base-2 exponential approximation.
///
/// Separates the integer and fractional parts of the exponent. The integer
/// part is computed via bit manipulation, while the fractional part uses a
/// Taylor-like polynomial expansion centered at 0.5.
#[inline]
pub fn exp2f(x: f32) -> f32 {
    if x < -126.0 {
        return 0.0;
    }
    if x > 126.0 {
        return f32::INFINITY;
    }
    let xf = x.floor();
    let exp_bits = ((127 + xf as i32) as u32) << 23;
    let ystep = f32::from_bits(exp_bits);

    let x1 = x - xf;
    let xt = x1 - 0.5;

    const C1: f32 = 0.980_258_17;
    const C2: f32 = 0.339_731_57;
    const C3: f32 = 0.078_494_66;
    const C4: f32 = 0.013_602_088;

    let ytaylor = SQRT_2 + xt * (C1 + xt * (C2 + xt * (C3 + xt * C4)));

    const M0: f32 = 0.999_944_3;
    const M1: f32 = 1.000_031_2;

    ystep * ytaylor * (M0 + (M1 - M0) * x1)
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

/// Fast `e^x` approximation via `2^(x * log2(e))`.
#[inline]
pub fn expf(x: f32) -> f32 {
    exp2f(x * LOG2_E)
}

/// Fast `e^x - 1` approximation.
///
/// Useful for small `x` where `e^x` is close to 1 and direct subtraction
/// would lose precision (though this fast version doesn't preserve that property).
#[inline]
pub fn expm1f(x: f32) -> f32 {
    exp2f(x * LOG2_E) - 1.0
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

/// Parabolic sine approximation.
///
/// Very fast but lower accuracy than [`sinf`]. Uses a single parabola
/// fitted to match sine at 0, ±π/2, and ±π.
#[inline]
pub fn par_sinf(x: f32) -> f32 {
    let x = modpi(x);
    0.405_284_73 * x * (PI - x.abs())
}

/// Parabolic cosine approximation.
///
/// Phase-shifted [`par_sinf`].
#[inline]
pub fn par_cosf(x: f32) -> f32 {
    par_sinf(x + 0.5 * PI)
}

/// Fast sine approximation using rational polynomial.
///
/// Higher accuracy than [`par_sinf`] but still significantly faster than
/// `std::f32::sin`. Uses a Padé-like rational approximation.
#[inline]
pub fn sinf(x: f32) -> f32 {
    let x = 4.0 * (x * (0.5 / PI) - (x * (0.5 / PI) + 0.75).floor() + 0.25).abs() - 1.0;
    let x = x * (PI / 2.0);

    const C1: f32 = 1.0;
    const C2: f32 = 445.0 / 12122.0;
    const C3: f32 = -(2363.0 / 18183.0);
    const C4: f32 = 601.0 / 872784.0;
    const C5: f32 = 12671.0 / 4363920.0;
    const C6: f32 = 121.0 / 16662240.0;

    let xx = x * x;
    let num = x * (C1 + xx * (C3 + xx * C5));
    let denom = 1.0 + xx * (C2 + xx * (C4 + xx * C6));
    num / denom
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
