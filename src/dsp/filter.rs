//! Biquad filter implementation for audio processing.
//!
//! Provides a second-order IIR (biquad) filter with multiple filter types and
//! coefficient caching for efficient real-time parameter modulation.
//!
//! # Filter Types
//!
//! | Type        | Description                                      |
//! |-------------|--------------------------------------------------|
//! | Lowpass     | Attenuates frequencies above cutoff              |
//! | Highpass    | Attenuates frequencies below cutoff              |
//! | Bandpass    | Passes frequencies near cutoff, attenuates rest  |
//! | Notch       | Attenuates frequencies near cutoff               |
//! | Allpass     | Passes all frequencies, shifts phase             |
//! | Peaking     | Boosts/cuts frequencies near cutoff              |
//! | Lowshelf    | Boosts/cuts frequencies below cutoff             |
//! | Highshelf   | Boosts/cuts frequencies above cutoff             |
//!
//! # Coefficient Formulas
//!
//! Based on Robert Bristow-Johnson's Audio EQ Cookbook.

use super::fastmath::{fast_tan, ftz, par_cosf, par_sinf, pow10};
use crate::types::FilterType;
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq)]
pub enum SvfMode {
    Lp,
    Hp,
    Bp,
}

/// State Variable Filter (Cytomic / Andy Simper topology) with coefficient caching.
///
/// Coefficients `g`, `k`, `a1`, `a2`, `a3` are recomputed only when `cutoff` or `q`
/// drift past a 0.1% relative threshold. Public `cutoff` is set by callers between
/// processing calls.
#[derive(Clone, Copy, Default)]
pub struct SvfState {
    /// Current cutoff frequency in Hz.
    pub cutoff: f32,
    ic1eq: f32,
    ic2eq: f32,
    cached_cutoff: f32,
    cached_q: f32,
    cached_g: f32,
    cached_k: f32,
    cached_a1: f32,
    cached_a2: f32,
    cached_a3: f32,
}

impl SvfState {
    #[inline]
    fn needs_recalc(&self, freq: f32, q: f32) -> bool {
        if self.cached_cutoff == 0.0 {
            return true;
        }
        let freq_delta = (freq - self.cached_cutoff).abs() / self.cached_cutoff;
        let q_delta = (q - self.cached_q).abs();
        freq_delta > 0.001 || q_delta > 0.001
    }

    #[inline]
    pub fn process(&mut self, input: f32, mode: SvfMode, q: f32, sr: f32) -> f32 {
        let freq = self.cutoff.clamp(1.0, sr * 0.45);
        let q = q.clamp(0.0, 1.0);
        if self.needs_recalc(freq, q) {
            let g = fast_tan(PI * freq / sr);
            let k = 2.0 * pow10(-2.0 * q);
            let a1 = 1.0 / (1.0 + g * (g + k));
            let a2 = g * a1;
            let a3 = g * a2;
            self.cached_cutoff = freq;
            self.cached_q = q;
            self.cached_g = g;
            self.cached_k = k;
            self.cached_a1 = a1;
            self.cached_a2 = a2;
            self.cached_a3 = a3;
        }

        let v3 = input - self.ic2eq;
        let v1 = self.cached_a1 * self.ic1eq + self.cached_a2 * v3;
        let v2 = self.ic2eq + self.cached_a2 * self.ic1eq + self.cached_a3 * v3;

        self.ic1eq = ftz(2.0 * v1 - self.ic1eq, 1e-20);
        self.ic2eq = ftz(2.0 * v2 - self.ic2eq, 1e-20);

        match mode {
            SvfMode::Lp => v2,
            SvfMode::Bp => v1,
            SvfMode::Hp => input - self.cached_k * v1 - v2,
        }
    }
}

/// Second-order IIR (biquad) filter using Transposed Direct Form II.
///
/// State variables `s1`, `s2` are integrator outputs, not signal history.
/// TDF2 needs only 2 state variables (vs 4 for Direct Form I) and behaves
/// more gracefully under per-sample coefficient modulation.
///
/// ```text
/// y    = b0 * x + s1
/// s1' = b1 * x - a1 * y + s2
/// s2' = b2 * x - a2 * y
/// ```
///
/// Coefficients are recalculated only when parameters change beyond a threshold
/// (0.1% for `freq`/`q`, 0.01 dB for `gain`), reducing CPU overhead during smooth
/// parameter automation.
#[derive(Clone, Copy)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    s1: f32,
    s2: f32,
    cached_freq: f32,
    cached_q: f32,
    cached_gain: f32,
    cached_filter_type: FilterType,
}

impl Default for Biquad {
    fn default() -> Self {
        Self {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            s1: 0.0,
            s2: 0.0,
            cached_freq: 0.0,
            cached_q: 0.0,
            cached_gain: 0.0,
            cached_filter_type: FilterType::Lowpass,
        }
    }
}

impl Biquad {
    #[inline]
    fn needs_recalc(&self, freq: f32, q: f32, gain: f32, filter_type: FilterType) -> bool {
        if self.cached_freq == 0.0 || filter_type != self.cached_filter_type {
            return true;
        }
        let freq_delta = (freq - self.cached_freq).abs() / self.cached_freq;
        let q_delta = (q - self.cached_q).abs() / self.cached_q;
        let gain_delta = (gain - self.cached_gain).abs();
        freq_delta > 0.001 || q_delta > 0.001 || gain_delta > 0.01
    }

    /// Processes a single sample through the filter.
    ///
    /// Convenience wrapper for [`Biquad::process_with_gain`] with `gain = 0.0`.
    #[inline]
    pub fn process(
        &mut self,
        input: f32,
        filter_type: FilterType,
        freq: f32,
        q: f32,
        sr: f32,
    ) -> f32 {
        self.process_with_gain(input, filter_type, freq, q, 0.0, sr)
    }

    /// Processes a single sample with gain parameter for shelving/peaking filters.
    ///
    /// Recalculates coefficients only when parameters change significantly.
    /// For lowpass and highpass, `q` is interpreted as resonance in dB.
    /// For other types, `q` is the Q factor directly.
    ///
    /// # Parameters
    ///
    /// - `input`: Input sample
    /// - `filter_type`: Type of filter response
    /// - `freq`: Cutoff/center frequency in Hz (clamped to `[1.0, sr*0.45]`)
    /// - `q`: Q factor or resonance in dB (clamped to `>= 0.05`)
    /// - `gain`: Boost/cut in dB (only used by peaking and shelving types)
    /// - `sr`: Sample rate in Hz
    #[inline]
    pub fn process_with_gain(
        &mut self,
        input: f32,
        filter_type: FilterType,
        freq: f32,
        q: f32,
        gain: f32,
        sr: f32,
    ) -> f32 {
        let freq = freq.clamp(1.0, sr * 0.45);
        let q = q.max(0.05);
        if self.needs_recalc(freq, q, gain, filter_type) {
            let (b0, b1, b2, a1, a2) = compute_biquad_coeffs(filter_type, freq, q, gain, sr);
            self.b0 = b0;
            self.b1 = b1;
            self.b2 = b2;
            self.a1 = a1;
            self.a2 = a2;
            self.cached_freq = freq;
            self.cached_q = q;
            self.cached_gain = gain;
            self.cached_filter_type = filter_type;
        }

        let y = self.b0 * input + self.s1;
        self.s1 = ftz(self.b1 * input - self.a1 * y + self.s2, 1e-20);
        self.s2 = ftz(self.b2 * input - self.a2 * y, 1e-20);
        y
    }
}

/// Computes biquad coefficients from filter parameters using the RBJ Audio EQ Cookbook.
///
/// Returns `(b0, b1, b2, a1, a2)` already normalized by `a0`. For Lowpass and Highpass,
/// `q` is interpreted as resonance in dB and converted via `10^(q/20)`; for all other
/// types `q` is the literal Q factor.
///
/// Preconditions: `freq` ∈ `[1.0, sr/2]`, `q >= 0.05`. Caller enforces.
#[inline]
fn compute_biquad_coeffs(
    filter_type: FilterType,
    freq: f32,
    q: f32,
    gain: f32,
    sr: f32,
) -> (f32, f32, f32, f32, f32) {
    let omega = 2.0 * PI * freq / sr;
    let sin_omega = par_sinf(omega);
    let cos_omega = par_cosf(omega);

    let q_linear = match filter_type {
        FilterType::Lowpass | FilterType::Highpass => pow10(q / 20.0),
        _ => q,
    };
    let alpha = sin_omega / (2.0 * q_linear);

    let (b0, b1, b2, a0, a1, a2) = match filter_type {
        FilterType::Lowpass => {
            let b1 = 1.0 - cos_omega;
            let b0 = b1 / 2.0;
            let b2 = b0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_omega;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::Highpass => {
            let b0 = (1.0 + cos_omega) / 2.0;
            let b1 = -(1.0 + cos_omega);
            let b2 = b0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_omega;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::Bandpass => {
            let b0 = sin_omega / 2.0;
            let b1 = 0.0;
            let b2 = -b0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_omega;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::Notch => {
            let b0 = 1.0;
            let b1 = -2.0 * cos_omega;
            let b2 = 1.0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_omega;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::Allpass => {
            let b0 = 1.0 - alpha;
            let b1 = -2.0 * cos_omega;
            let b2 = 1.0 + alpha;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_omega;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::Peaking => {
            let a = pow10(gain / 40.0);
            let b0 = 1.0 + alpha * a;
            let b1 = -2.0 * cos_omega;
            let b2 = 1.0 - alpha * a;
            let a0 = 1.0 + alpha / a;
            let a1 = -2.0 * cos_omega;
            let a2 = 1.0 - alpha / a;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::Lowshelf => {
            let a = pow10(gain / 40.0);
            let sqrt2_a_alpha = 2.0 * a.sqrt() * alpha;
            let am1_cos = (a - 1.0) * cos_omega;
            let ap1_cos = (a + 1.0) * cos_omega;
            let b0 = a * ((a + 1.0) - am1_cos + sqrt2_a_alpha);
            let b1 = 2.0 * a * ((a - 1.0) - ap1_cos);
            let b2 = a * ((a + 1.0) - am1_cos - sqrt2_a_alpha);
            let a0 = (a + 1.0) + am1_cos + sqrt2_a_alpha;
            let a1 = -2.0 * ((a - 1.0) + ap1_cos);
            let a2 = (a + 1.0) + am1_cos - sqrt2_a_alpha;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::Highshelf => {
            let a = pow10(gain / 40.0);
            let sqrt2_a_alpha = 2.0 * a.sqrt() * alpha;
            let am1_cos = (a - 1.0) * cos_omega;
            let ap1_cos = (a + 1.0) * cos_omega;
            let b0 = a * ((a + 1.0) + am1_cos + sqrt2_a_alpha);
            let b1 = -2.0 * a * ((a - 1.0) + ap1_cos);
            let b2 = a * ((a + 1.0) + am1_cos - sqrt2_a_alpha);
            let a0 = (a + 1.0) - am1_cos + sqrt2_a_alpha;
            let a1 = 2.0 * ((a - 1.0) - ap1_cos);
            let a2 = (a + 1.0) - am1_cos - sqrt2_a_alpha;
            (b0, b1, b2, a0, a1, a2)
        }
    };

    (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
}
