//! Phaser effect using cascaded notch filters.
//!
//! Creates the sweeping, hollow sound by modulating two notch filters
//! with an LFO. The notches are offset by ~282 Hz for a richer effect.

use crate::dsp::{exp2f, Biquad, Phasor};
use crate::types::FilterType;

/// Frequency offset between the two notch filters (Hz).
const NOTCH_OFFSET: f32 = 282.0;

/// Two-stage phaser with LFO modulation.
#[derive(Clone, Copy, Default)]
pub struct Phaser {
    notch1: Biquad,
    notch2: Biquad,
    lfo: Phasor,
}

impl Phaser {
    /// Processes one sample.
    ///
    /// - `rate`: LFO speed in Hz
    /// - `depth`: Notch resonance (higher = more pronounced, max ~0.95)
    /// - `center`: Base frequency in Hz
    /// - `sweep`: Modulation range in cents (1200 = ±1 octave)
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        input: f32,
        rate: f32,
        depth: f32,
        center: f32,
        sweep: f32,
        sr: f32,
        isr: f32,
    ) -> f32 {
        let lfo_val = self.lfo.sine(rate, isr);
        let q = 2.0 - (depth * 2.0).min(1.9);
        let detune = exp2f(lfo_val * sweep * (1.0 / 1200.0));

        let max_freq = sr * 0.45;
        let freq1 = (center * detune).clamp(20.0, max_freq);
        let freq2 = ((center + NOTCH_OFFSET) * detune).clamp(20.0, max_freq);

        let out = self.notch1.process(input, FilterType::Notch, freq1, q, sr);
        self.notch2.process(out, FilterType::Notch, freq2, q, sr)
    }
}
