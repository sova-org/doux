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

use super::fastmath::{ftz, par_cosf, par_sinf, pow10};
use crate::types::FilterType;
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq)]
pub enum SvfMode {
    Lp,
    Hp,
    Bp,
}

#[derive(Clone, Copy)]
pub struct Svf {
    ic1eq: f32,
    ic2eq: f32,
}

impl Default for Svf {
    fn default() -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }
}

impl Svf {
    #[inline]
    pub fn tick(&mut self, input: f32, g: f32, k: f32, mode: SvfMode) -> f32 {
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = ftz(2.0 * v1 - self.ic1eq, 1e-20);
        self.ic2eq = ftz(2.0 * v2 - self.ic2eq, 1e-20);

        match mode {
            SvfMode::Lp => v2,
            SvfMode::Bp => v1,
            SvfMode::Hp => input - k * v1 - v2,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SvfState {
    pub cutoff: f32,
    stage: Svf,
    cached_cutoff: f32,
    cached_q: f32,
    cached_g: f32,
    cached_k: f32,
}

impl Default for SvfState {
    fn default() -> Self {
        Self {
            cutoff: 0.0,
            stage: Svf::default(),
            cached_cutoff: f32::NAN,
            cached_q: f32::NAN,
            cached_g: 0.0,
            cached_k: 0.0,
        }
    }
}

impl SvfState {
    #[inline]
    pub fn process(&mut self, input: f32, mode: SvfMode, q: f32, sr: f32) -> f32 {
        let freq = self.cutoff.clamp(1.0, sr * 0.45);
        let q = q.clamp(0.0, 1.0);
        if self.cached_cutoff != freq || self.cached_q != q {
            self.cached_cutoff = freq;
            self.cached_q = q;
            self.cached_g = (PI * freq / sr).tan();
            self.cached_k = 2.0 * pow10(-2.0 * q);
        }
        self.stage.tick(input, self.cached_g, self.cached_k, mode)
    }
}

/// Second-order IIR (biquad) filter with coefficient caching.
///
/// Implements the standard Direct Form I difference equation:
///
/// ```text
/// y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] - a1*y[n-1] - a2*y[n-2]
/// ```
///
/// Coefficients are recalculated only when parameters change beyond a threshold,
/// reducing CPU overhead during smooth parameter automation.
#[derive(Clone, Copy)]
pub struct Biquad {
    // Feedforward coefficients (numerator)
    b0: f32,
    b1: f32,
    b2: f32,
    // Feedback coefficients (denominator, negated)
    a1: f32,
    a2: f32,
    // Input delay line
    x1: f32,
    x2: f32,
    // Output delay line
    y1: f32,
    y2: f32,
    // Cached parameters for change detection
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
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            cached_freq: 0.0,
            cached_q: 0.0,
            cached_gain: 0.0,
            cached_filter_type: FilterType::Lowpass,
        }
    }
}

impl Biquad {
    /// Checks if parameters have changed enough to warrant coefficient recalculation.
    ///
    /// Uses relative thresholds: 0.1% for frequency and Q, 0.01 dB for gain.
    #[inline]
    fn needs_recalc(&self, freq: f32, q: f32, gain: f32, filter_type: FilterType) -> bool {
        if filter_type != self.cached_filter_type {
            return true;
        }
        let freq_delta = (freq - self.cached_freq).abs() / self.cached_freq.max(1.0);
        let q_delta = (q - self.cached_q).abs() / self.cached_q.max(0.1);
        let gain_delta = (gain - self.cached_gain).abs();
        freq_delta > 0.001 || q_delta > 0.001 || gain_delta > 0.01
    }

    /// Processes a single sample through the filter.
    ///
    /// Convenience wrapper for [`Biquad::process_with_gain`] with `gain = 0.0`.
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
    /// - `freq`: Cutoff/center frequency in Hz
    /// - `q`: Q factor or resonance (interpretation depends on filter type)
    /// - `gain`: Boost/cut in dB (only used by peaking and shelving types)
    /// - `sr`: Sample rate in Hz
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
        if self.needs_recalc(freq, q, gain, filter_type) {
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

            self.b0 = b0 / a0;
            self.b1 = b1 / a0;
            self.b2 = b2 / a0;
            self.a1 = a1 / a0;
            self.a2 = a2 / a0;

            self.cached_freq = freq;
            self.cached_q = q;
            self.cached_gain = gain;
            self.cached_filter_type = filter_type;
        }

        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
}

/// Multi-stage filter state for cascaded biquad processing.
///
/// Contains up to 4 biquad stages for steeper filter slopes (up to 48 dB/octave).
#[derive(Clone, Copy, Default)]
pub struct FilterState {
    /// Current cutoff frequency in Hz.
    pub cutoff: f32,
    /// Cascaded biquad filter stages.
    pub biquads: [Biquad; 4],
}
