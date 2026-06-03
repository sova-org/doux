//! Waveshaping distortion effects.
//!
//! Stateful (first-order antiderivative anti-aliasing, Parker et al. DAFx-16):
//! - [`Saturate`]: soft-knee saturation with baked asymmetry + drive comp.
//! - [`Fold`]: reflective triangle wavefolder.
//! - [`Wrap`]: phase wrapping.
//!
//! Utility:
//! - [`DcBlocker`]: single-pole DC-removal HPF (~20 Hz corner).
//!
//! ADAA replaces `y = f(x)` with `y = (F(x) − F(x₋₁)) / (x − x₋₁)` where
//! `F` is the antiderivative of `f`. When consecutive inputs are too close
//! we fall back to a midpoint evaluation `f((x + x₋₁) / 2)` to dodge 0/0.
//! Cost is ~2 extra FLOPs per sample. On the smooth saturator/folder this is
//! ~2× oversampling; on the jump-discontinuous wrapper it adds one order of
//! alias rolloff — it attenuates the alias energy but does not remove it.

use crate::dsp::{exp2f, log2f, powf};
use crate::types::{ModuleGroup, ModuleInfo, ParamInfo, StereoFrame};

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
            description: "triangle wavefolding amount",
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

/// Fold drive depth: `d = 2^(amount·FOLD_DEPTH)` ∈ `[1, 2^FOLD_DEPTH]`. Caps
/// fold density to bound aliasing (a full-scale signal sees ≈`d` reflections).
const FOLD_DEPTH: f32 = 3.0;
/// Fold output makeup compensation fraction: `0` = none, `1` = full `1/d`.
/// Cancels the drive's small-signal level rise without crushing full-scale.
const FOLD_COMP: f32 = 0.5;

/// Baked-in pre-shaper DC bias. Breaks the otherwise odd-symmetric curve to add
/// even harmonics (subtle tube-like warmth); the trailing [`DcBlocker`] stage
/// removes the resulting DC. Tune by ear.
const DISTORT_BIAS: f32 = 0.05;
/// Drive-loudness compensation exponent: output is scaled by `(1+k)^-p` so
/// turning up drive changes timbre, not loudness. `p = 0.12` tracks the curve's
/// measured RMS gain (≈ 1.0 → 1.33 over `k = 0 → 20`), not the steeper
/// small-signal slope (which would over-attenuate). Tune by ear.
const DISTORT_COMP_P: f32 = 0.12;

/// Soft-knee saturator with first-order ADAA, baked asymmetry, and
/// drive-compensated output gain.
///
/// Curve `f(x) = (1+k)·x / (1 + k·|x|)`, `k = amount`. Antiderivative (even,
/// `F' = f`): `F(x) = (1+k)/k·|x| − (1+k)/k²·ln(1 + k·|x|)`. Below `k ≈ 0` the
/// curve is the identity, so the `1/k` terms are bypassed.
#[derive(Clone, Copy, Default)]
pub struct Saturate {
    state: AdaaState,
    last_k: f32,
    comp: f32,
    fb: f32,
}

impl Saturate {
    /// Refresh the per-`k` constants (drive comp and the bias re-centering
    /// offset) only when `k` changes — keeps the per-sample path off the `powf`.
    #[inline]
    fn refresh(&mut self, k: f32) {
        if k != self.last_k {
            self.comp = powf(1.0 + k, -DISTORT_COMP_P);
            self.fb = sat_curve(DISTORT_BIAS, k);
            self.last_k = k;
        }
    }

    #[inline]
    pub fn process(&mut self, x: f32, amount: f32, postgain: f32) -> f32 {
        let k = amount.max(0.0);
        if k < 1.0e-4 {
            return x * postgain;
        }
        self.refresh(k);
        let g = postgain * self.comp;
        (self.state.step(x + DISTORT_BIAS, k, sat_anti, sat_curve) - self.fb) * g
    }

    #[inline]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        amount: f32,
        postgain: f32,
    ) {
        let k = amount.max(0.0);
        if k < 1.0e-4 {
            for slot in buf.iter_mut().take(n) {
                slot[ch] *= postgain;
            }
            return;
        }
        self.refresh(k);
        let g = postgain * self.comp;
        for slot in buf.iter_mut().take(n) {
            let y = self.state.step(slot[ch] + DISTORT_BIAS, k, sat_anti, sat_curve) - self.fb;
            slot[ch] = y * g;
        }
    }
}

/// Unity-gain saturation curve `(1+k)·x / (1 + k·|x|)`; also the ADAA midpoint.
#[inline]
fn sat_curve(x: f32, k: f32) -> f32 {
    (1.0 + k) * x / (1.0 + k * x.abs())
}

/// Even antiderivative of [`sat_curve`]: `(1+k)/k·|x| − (1+k)/k²·ln(1 + k·|x|)`.
#[inline]
fn sat_anti(x: f32, k: f32) -> f32 {
    let a = x.abs();
    let c = (1.0 + k) / k;
    c * a - (c / k) * (log2f(1.0 + k * a) * std::f32::consts::LN_2)
}

/// First-order ADAA state. Caller supplies the nonlinearity's antiderivative
/// `F` and a midpoint evaluator `f((x + x₋₁)/2)` used when `|Δx|` is too small
/// for the difference quotient. Param-change detection re-evaluates `F(x₋₁)`
/// under the new curve to keep the next step mathematically consistent.
#[derive(Clone, Copy, Default)]
struct AdaaState {
    x_prev: f32,
    f_prev: f32,
    last_k: f32,
}

impl AdaaState {
    #[inline]
    fn step(
        &mut self,
        x: f32,
        k: f32,
        antideriv: impl Fn(f32, f32) -> f32,
        midpoint: impl Fn(f32, f32) -> f32,
    ) -> f32 {
        if k != self.last_k {
            self.f_prev = antideriv(self.x_prev, k);
            self.last_k = k;
        }
        let f_x = antideriv(x, k);
        let dx = x - self.x_prev;
        let y = if dx.abs() < ADAA_EPS {
            midpoint((x + self.x_prev) * 0.5, k)
        } else {
            (f_x - self.f_prev) / dx
        };
        self.x_prev = x;
        self.f_prev = f_x;
        y
    }
}

/// Reflective triangle wavefolder: `y = g · tri(d · x)` with drive
/// `d = 2^(amount · FOLD_DEPTH)` and amount-only makeup `g`. The triangle has
/// slope ±1 and is bounded to ±1 for every drive, so there is no small-signal
/// gain blow-up; `d` sets fold density, `g` holds perceived level. Reuses the
/// same first-order ADAA engine as [`Wrap`] — `tri` is piecewise-linear, which
/// ADAA band-limits well — and `tri` is odd, so it introduces no DC.
#[derive(Clone, Copy, Default)]
pub struct Fold {
    state: AdaaState,
}

impl Fold {
    #[inline]
    pub fn process(&mut self, x: f32, amount: f32) -> f32 {
        let d = exp2f(amount * FOLD_DEPTH);
        let g = exp2f(-amount * FOLD_DEPTH * FOLD_COMP);
        g * self.state.step(x, d, antideriv_tri, |x, d| tri(d * x))
    }

    #[inline]
    pub fn process_block(&mut self, buf: &mut [StereoFrame], n: usize, ch: usize, amount: f32) {
        let d = exp2f(amount * FOLD_DEPTH);
        let g = exp2f(-amount * FOLD_DEPTH * FOLD_COMP);
        for slot in buf.iter_mut().take(n) {
            slot[ch] = g * self.state.step(slot[ch], d, antideriv_tri, |x, d| tri(d * x));
        }
    }
}

/// Reflective triangle: period-4 triangle wave, slope ±1, `|tri| ≤ 1`, odd.
#[inline]
fn tri(v: f32) -> f32 {
    1.0 - ((v + 1.0).rem_euclid(4.0) - 2.0).abs()
}

/// Antiderivative of `tri(d · x)` w.r.t. `x`, used by ADAA. With
/// `p = rem_euclid(d·x + 1, 4)`, `H(p)` is continuous across every corner
/// (`H(2⁻) = H(2⁺) = 0`, period-wrap to 0), so each period integrates to zero —
/// the same property [`antideriv_wrap`] relies on, which is why ADAA works here.
#[inline]
fn antideriv_tri(x: f32, d: f32) -> f32 {
    let p = (d * x + 1.0).rem_euclid(4.0);
    let h = if p < 2.0 {
        p * p * 0.5 - p
    } else {
        3.0 * p - p * p * 0.5 - 4.0
    };
    h / d
}

/// Phase wrapper: `f(x) = ((k·x + 1) rem 2) − 1` with `k = 1 + wraps`, plus a
/// baked pre-wrap [`WRAP_BIAS`] offset for even-harmonic character.
///
/// Piecewise-linear sawtooth in `x`; the naive form aliases severely.
/// Antiderivative used by ADAA: `F(x) = (v − 1)² / (2k)` with `v = wrap2(k·x+1)`.
/// `F` is continuous across the discontinuities of `f` (each period integrates
/// to zero), which is exactly why ADAA works.
#[derive(Clone, Copy, Default)]
pub struct Wrap {
    state: AdaaState,
}

impl Wrap {
    #[inline]
    pub fn process(&mut self, x: f32, wraps: f32) -> f32 {
        let k = 1.0 + wraps;
        let inv2k = 0.5 / k;
        self.state.step(
            x + WRAP_BIAS,
            k,
            |u, k| {
                let d = wrap2(k * u + 1.0) - 1.0;
                d * d * inv2k
            },
            |u, k| wrap2(k * u + 1.0) - 1.0,
        )
    }

    #[inline]
    pub fn process_block(&mut self, buf: &mut [StereoFrame], n: usize, ch: usize, wraps: f32) {
        let k = 1.0 + wraps;
        let inv2k = 0.5 / k;
        for slot in buf.iter_mut().take(n) {
            slot[ch] = self.state.step(
                slot[ch] + WRAP_BIAS,
                k,
                |u, k| {
                    let d = wrap2(k * u + 1.0) - 1.0;
                    d * d * inv2k
                },
                |u, k| wrap2(k * u + 1.0) - 1.0,
            );
        }
    }
}

/// Baked-in pre-wrap DC bias: breaks the wrapper's odd symmetry to add even
/// harmonics (subtle "thicker" character); the trailing [`DcBlocker`] stage
/// removes the resulting DC. Tune by ear.
const WRAP_BIAS: f32 = 0.04;

/// Wrap `u` into `[0, 2)` without the `rem_euclid` sign-fixup. Bit-exact to
/// `u.rem_euclid(2.0)` over the wrapper's input range.
#[inline]
fn wrap2(u: f32) -> f32 {
    u - 2.0 * (u * 0.5).floor()
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

    #[inline]
    pub fn process_block(&mut self, buf: &mut [StereoFrame], n: usize, ch: usize) {
        const R: f32 = 0.9995;
        for slot in buf.iter_mut().take(n) {
            let x = slot[ch];
            let y = x - self.x_prev + R * self.y_prev;
            self.x_prev = x;
            self.y_prev = y;
            slot[ch] = y;
        }
    }
}
