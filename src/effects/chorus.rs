//! Multi-voice chorus effect with stereo modulation.
//!
//! Creates a shimmering, widened sound by mixing the dry signal with multiple
//! delayed copies whose delay times are modulated by LFOs. Each voice uses a
//! different LFO phase, and left/right channels are modulated in opposite
//! directions for stereo spread.
//!
//! # Signal Flow
//!
//! ```text
//! L+R → mono → delay line ─┬─ voice 0 (LFO phase 0°)   ─┬─→ L
//!                          ├─ voice 1 (LFO phase 120°) ─┤
//!                          └─ voice 2 (LFO phase 240°) ─┴─→ R
//! ```
//!
//! The three voices are phase-offset by 120° to avoid reinforcement artifacts.
//! Left and right taps use opposite modulation polarity for stereo width.

use crate::dsp::{ms_to_samples, DelayLine, Phasor};
use crate::effects::Lag;
use crate::types::{ModuleGroup, ModuleInfo, ParamInfo, StereoFrame};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "chorus",
    description: "3-voice modulated delay with stereo spread",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "chorus",
            aliases: &["chorusrate"],
            description: "LFO rate in Hz (0 = bypass)",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "chorusdepth",
            aliases: &[],
            description: "modulation intensity",
            default: "0.35",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "chorusdelay",
            aliases: &[],
            description: "base delay time in ms",
            default: "25.0",
            min: 0.0,
            max: 100.0,
        },
    ],
};

/// Max chorus delay time in ms. Sized for `MAX_SAMPLE_RATE`.
const MAX_DELAY_MS: usize = 50;
const BUFFER_SIZE: usize =
    (crate::types::MAX_SAMPLE_RATE * MAX_DELAY_MS / 1000).next_power_of_two();

/// Number of chorus voices (phase-offset delay taps).
const VOICES: usize = 3;

/// Multi-voice stereo chorus effect.
///
/// Uses a circular delay buffer with three LFO-modulated tap points.
/// The LFOs are phase-offset by 1/3 cycle (120°) to create smooth,
/// non-pulsing modulation.
#[derive(Clone, Copy)]
pub struct Chorus {
    delay: DelayLine<BUFFER_SIZE>,
    lfo: [Phasor; VOICES],
    delay_lag: Lag,
}

impl Default for Chorus {
    fn default() -> Self {
        let mut lfo = [Phasor::default(); VOICES];
        for (i, l) in lfo.iter_mut().enumerate() {
            l.phase = i as f32 / VOICES as f32;
        }
        Self {
            delay: DelayLine::default(),
            lfo,
            delay_lag: Lag::default(),
        }
    }
}

/// One-pole smoothing time for the base delay (seconds): fast enough to feel
/// instant, slow enough to suppress clicks when `chorusdelay` jumps.
const DELAY_SMOOTH_SECS: f32 = 0.02;

impl Chorus {
    /// Processes one stereo sample through the chorus.
    ///
    /// # Parameters
    ///
    /// - `left`, `right`: Input stereo sample
    /// - `rate`: LFO frequency in Hz (typical: 0.5-3.0)
    /// - `depth`: Modulation intensity `[0.0, 1.0]`
    /// - `delay_ms`: Base delay time in milliseconds (typical: 10-30)
    /// - `sr`: Sample rate in Hz
    /// - `isr`: Inverse sample rate (1.0 / sr)
    ///
    /// # Returns
    ///
    /// Stereo output `[left, right]` with 50/50 dry/wet mix (equal power).
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        left: f32,
        right: f32,
        rate: f32,
        depth: f32,
        delay_ms: f32,
        sr: f32,
        isr: f32,
    ) -> [f32; 2] {
        let depth = depth.clamp(0.0, 1.0);
        let smoothed_delay = self.delay_lag.update(delay_ms, 1.0, sr * DELAY_SMOOTH_SECS);
        let mod_range = smoothed_delay * 0.8;

        let mono = (left + right) * 0.5;
        self.delay.write(mono);

        let mut out_l = 0.0_f32;
        let mut out_r = 0.0_f32;

        let min_delay = 1.5;
        let max_delay = 50.0_f32.min((BUFFER_SIZE as f32 - 2.0) * 1000.0 / sr);

        for v in 0..VOICES {
            let lfo = self.lfo[v].sine(rate, isr);

            let modulation = depth * mod_range * lfo;
            let dly_l = (smoothed_delay + modulation).clamp(min_delay, max_delay);
            let dly_r = (smoothed_delay - modulation).clamp(min_delay, max_delay);

            let samp_l = ms_to_samples(dly_l, sr).clamp(2.0, BUFFER_SIZE as f32 - 3.0);
            let samp_r = ms_to_samples(dly_r, sr).clamp(2.0, BUFFER_SIZE as f32 - 3.0);

            out_l += self.delay.read_cubic(samp_l);
            out_r += self.delay.read_cubic(samp_r);
        }

        out_l /= VOICES as f32;
        out_r /= VOICES as f32;

        const MIX: f32 = std::f32::consts::FRAC_1_SQRT_2;
        [left * MIX + out_l * MIX, right * MIX + out_r * MIX]
    }

    /// Block-rate stereo processing. Mirrors [`Self::process`] across `n` frames,
    /// hoisting invariant constants (delay bounds, buffer cap, mix) out of the
    /// loop. The smoothed base delay and per-voice LFO stay per-sample for
    /// modulation fidelity. Body is inlined rather than calling [`Self::process`]
    /// to keep those hoists.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        rate: f32,
        depth: f32,
        delay_ms: f32,
        sr: f32,
        isr: f32,
    ) {
        let depth = depth.clamp(0.0, 1.0);
        let min_delay = 1.5_f32;
        let max_delay = 50.0_f32.min((BUFFER_SIZE as f32 - 2.0) * 1000.0 / sr);
        let buf_cap = BUFFER_SIZE as f32 - 3.0;
        let lag_unit = sr * DELAY_SMOOTH_SECS;
        const MIX: f32 = std::f32::consts::FRAC_1_SQRT_2;
        for slot in buf.iter_mut().take(n) {
            let in_l = slot[0];
            let in_r = slot[1];
            let mono = (in_l + in_r) * 0.5;
            self.delay.write(mono);
            let smoothed_delay = self.delay_lag.update(delay_ms, 1.0, lag_unit);
            let mod_range = smoothed_delay * 0.8;
            let mut out_l = 0.0_f32;
            let mut out_r = 0.0_f32;
            for v in 0..VOICES {
                let lfo = self.lfo[v].sine(rate, isr);
                let modulation = depth * mod_range * lfo;
                let dly_l = (smoothed_delay + modulation).clamp(min_delay, max_delay);
                let dly_r = (smoothed_delay - modulation).clamp(min_delay, max_delay);
                let samp_l = ms_to_samples(dly_l, sr).clamp(2.0, buf_cap);
                let samp_r = ms_to_samples(dly_r, sr).clamp(2.0, buf_cap);
                out_l += self.delay.read_cubic(samp_l);
                out_r += self.delay.read_cubic(samp_r);
            }
            out_l /= VOICES as f32;
            out_r /= VOICES as f32;
            slot[0] = in_l * MIX + out_l * MIX;
            slot[1] = in_r * MIX + out_r * MIX;
        }
    }
}
