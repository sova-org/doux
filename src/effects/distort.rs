//! Waveshaping distortion effects.
//!
//! Stateless:
//! - [`distort`]: soft saturation (`x / (1 + k|x|)`, tube-like warmth).
//!
//! Stateful (first-order antiderivative anti-aliasing, Parker et al. DAFx-16):
//! - [`Fold`]: sine wavefolder.
//! - [`Wrap`]: phase wrapping.
//!
//! Utility:
//! - [`DcBlocker`]: single-pole DC-removal HPF (~20 Hz corner).
//!
//! ADAA replaces `y = f(x)` with `y = (F(x) − F(x₋₁)) / (x − x₋₁)` where
//! `F` is the antiderivative of `f`. When consecutive inputs are too close
//! we fall back to a midpoint evaluation `f((x + x₋₁) / 2)` to dodge 0/0.
//! Cost is ~2 extra FLOPs per sample; perceptually equivalent to 2× over-
//! sampling on smooth nonlinearities and much better than that on the
//! piecewise-linear wrapper.

use crate::dsp::{cosf, exp2f, sinf};
use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "distort",
    description: "Waveshaping distortion (saturation, wavefolding, phase wrapping)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "distort",
            aliases: &[],
            description:
                "soft saturation amount (unbounded — saturates to hard clip at high values)",
            default: "0.0",
            min: 0.0,
            max: f32::MAX,
        },
        ParamInfo {
            name: "fold",
            aliases: &[],
            description: "sine wavefolding amount",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "wrap",
            aliases: &[],
            description: "phase wrapping amount",
            default: "0.0",
            min: 0.0,
            max: 10.0,
        },
        ParamInfo {
            name: "distortvol",
            aliases: &[],
            description: "output volume compensation",
            default: "1.0",
            min: 0.0,
            max: 2.0,
        },
    ],
};

/// Guard threshold for the ADAA 0/0 case. Below this, fall back to midpoint.
const ADAA_EPS: f32 = 1.0e-5;

/// Soft-knee saturation with adjustable drive.
///
/// `(1+k)·x / (1 + k·|x|)` with `k = amount` (linear drive). Bounded and
/// smooth — no anti-aliasing needed. Range is spread across the whole axis
/// instead of piling up exponentially in the first few units.
#[inline]
pub fn distort(input: f32, amount: f32, postgain: f32) -> f32 {
    let k = amount.max(0.0);
    ((1.0 + k) * input / (1.0 + k * input.abs())) * postgain
}

/// Sine wavefolder: `f(x) = sin(x · g · π/2)` with `g = 2^(amt·4)`.
///
/// Antiderivative used by ADAA: `F(x) = −cos(x · g · π/2) / (g · π/2)`.
#[derive(Clone, Copy, Default)]
pub struct Fold {
    x_prev: f32,
    f_prev: f32,
    last_k: f32,
}

impl Fold {
    #[inline]
    pub fn process(&mut self, x: f32, amount: f32) -> f32 {
        let gain = exp2f(amount * 4.0);
        let k = gain * std::f32::consts::FRAC_PI_2;

        // Parameter change: re-evaluate F(x_prev) under the new curve so the
        // next difference stays mathematically consistent.
        if k != self.last_k {
            self.f_prev = -cosf(self.x_prev * k) / k;
            self.last_k = k;
        }

        let f_x = -cosf(x * k) / k;
        let dx = x - self.x_prev;
        let y = if dx.abs() < ADAA_EPS {
            sinf((x + self.x_prev) * 0.5 * k)
        } else {
            (f_x - self.f_prev) / dx
        };

        self.x_prev = x;
        self.f_prev = f_x;
        y
    }
}

/// Phase wrapper: `f(x) = ((k·x + 1) rem 2) − 1` with `k = 1 + wraps`.
///
/// Piecewise-linear sawtooth in `x`; the naive form aliases severely.
/// Antiderivative used by ADAA: `F(x) = (v − 1)² / (2k)` with
/// `v = rem_euclid(k·x + 1, 2)`. `F` is continuous across the discontinuities
/// of `f` (each period integrates to zero), which is exactly why ADAA works.
#[derive(Clone, Copy, Default)]
pub struct Wrap {
    x_prev: f32,
    f_prev: f32,
    last_k: f32,
}

impl Wrap {
    #[inline]
    pub fn process(&mut self, x: f32, wraps: f32) -> f32 {
        let k = 1.0 + wraps;

        if k != self.last_k {
            self.f_prev = antideriv_wrap(self.x_prev, k);
            self.last_k = k;
        }

        let f_x = antideriv_wrap(x, k);
        let dx = x - self.x_prev;
        let y = if dx.abs() < ADAA_EPS {
            let m = (x + self.x_prev) * 0.5;
            (k * m + 1.0).rem_euclid(2.0) - 1.0
        } else {
            (f_x - self.f_prev) / dx
        };

        self.x_prev = x;
        self.f_prev = f_x;
        y
    }
}

#[inline]
fn antideriv_wrap(x: f32, k: f32) -> f32 {
    let v = (k * x + 1.0).rem_euclid(2.0);
    let d = v - 1.0;
    d * d / (2.0 * k)
}

/// First-order DC blocker. `y = x − x₋₁ + R · y₋₁` with `R = 0.9995`
/// (≈ 20 Hz corner at 48 kHz). Cheap; removes the DC creep introduced by
/// asymmetric drive + modulation upstream.
#[derive(Clone, Copy, Default)]
pub struct DcBlocker {
    x_prev: f32,
    y_prev: f32,
}

impl DcBlocker {
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        const R: f32 = 0.9995;
        let y = x - self.x_prev + R * self.y_prev;
        self.x_prev = x;
        self.y_prev = y;
        y
    }
}
