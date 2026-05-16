//! Phaser effect using cascaded notch filters.
//!
//! Creates the sweeping, hollow sound by modulating two notch filters
//! with an LFO. The notches are offset by ~282 Hz for a richer effect.

use crate::dsp::{exp2f, Biquad, Phasor};
use crate::types::{FilterType, ModuleGroup, ModuleInfo, ParamInfo, StereoFrame};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "phaser",
    description: "Two-stage notch filter with LFO modulation",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "phaser",
            aliases: &["phaserrate"],
            description: "LFO rate in Hz (0 = bypass)",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "phaserdepth",
            aliases: &[],
            description: "notch resonance",
            default: "0.75",
            min: 0.0,
            max: 0.95,
        },
        ParamInfo {
            name: "phasersweep",
            aliases: &[],
            description: "modulation range in cents",
            default: "1200.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "phasercenter",
            aliases: &[],
            description: "base center frequency in Hz",
            default: "800.0",
            min: 0.0,
            max: 20000.0,
        },
    ],
};

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
    /// Processes a block of stereo frames in place on channel `ch`.
    ///
    /// `q`, `max_freq`, `sweep_scaled`, and `freq2_offset` hoist to block entry;
    /// LFO ticks per sample (modulates notch frequency), and the biquad
    /// `needs_recalc` threshold gates per-sample coefficient recompute.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        rate: f32,
        depth: f32,
        center: f32,
        sweep: f32,
        sr: f32,
        isr: f32,
    ) {
        let q = 2.0 - (depth * 2.0).min(1.9);
        let max_freq = sr * 0.45;
        let center2 = center + NOTCH_OFFSET;
        for slot in buf.iter_mut().take(n) {
            let lfo_val = self.lfo.sine(rate, isr);
            // Preserve legacy left-to-right product order: ((lfo · sweep) · 1/1200).
            let detune = exp2f(lfo_val * sweep * (1.0 / 1200.0));
            let freq1 = (center * detune).clamp(20.0, max_freq);
            let freq2 = (center2 * detune).clamp(20.0, max_freq);
            let input = slot[ch];
            let out = self.notch1.process(input, FilterType::Notch, freq1, q, sr);
            slot[ch] = self.notch2.process(out, FilterType::Notch, freq2, q, sr);
        }
    }
}
