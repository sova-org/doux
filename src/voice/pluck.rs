//! Karplus-Strong plucked string for the `pluck` source.

use crate::dsp::{exp2f, DelayLine};
use crate::effects::DcBlocker;

use super::Voice;

/// Delay line length (power of two for the circular mask). Max usable delay
/// of `PLUCK_BUF - 4` samples puts the pitch floor at ≈ 5.9 Hz @ 48 kHz and
/// ≈ 11.7 Hz @ 96 kHz. 32 KB per voice, allocated once at construction.
pub(crate) const PLUCK_BUF: usize = 8192;

/// Largest delay where all four cubic-read taps stay within written history.
const MAX_DELAY: f32 = (PLUCK_BUF - 4) as f32;

#[derive(Clone)]
pub(crate) struct PluckState {
    delay: DelayLine<PLUCK_BUF>,
    /// One-pole lowpass state inside the feedback loop (string damping).
    damp: f32,
    /// One-pole lowpass coloring the excitation burst.
    exciter_lp: f32,
    dc: DcBlocker,
    /// Remaining noise-burst samples (one period at note-on).
    burst_remaining: u32,
    /// Set by the first `run_pluck` call of a note; cleared in `Voice::reset`.
    /// Priming (re-zeroing the line) happens lazily here so `reset` stays O(1)
    /// for every other source.
    pub(super) primed: bool,
}

impl Default for PluckState {
    fn default() -> Self {
        Self {
            delay: DelayLine::default(),
            damp: 0.0,
            exciter_lp: 0.0,
            dc: DcBlocker::default(),
            burst_remaining: 0,
            primed: false,
        }
    }
}

impl Voice {
    /// One Karplus-Strong sample: noise burst into a damped, tuned feedback
    /// loop. The delay is retuned every sample so vibrato and per-sample freq
    /// modulation bend the string continuously.
    ///
    /// Param mapping: `timbre` → loop damping (brightness), `harmonics` →
    /// loop gain (sustain), `morph` → excitation color (dark thud to snap).
    pub(super) fn run_pluck(&mut self, freq: f32, isr: f32) -> f32 {
        let period = 1.0 / (freq * isr).max(1.0e-6);

        if !self.pluck.primed {
            *self.pluck = PluckState::default();
            self.pluck.burst_remaining = period.clamp(2.0, MAX_DELAY) as u32;
            self.pluck.primed = true;
        }

        let timbre = self.params.timbre.clamp(0.0, 1.0);
        let harmonics = self.params.harmonics.clamp(0.0, 1.0);
        let morph = self.params.morph.clamp(0.0, 1.0);

        // Loop damping: timbre 0 = dark/fast decay, 1 = bright/ringing.
        let a = 0.04 + 0.95 * timbre * timbre;
        // Compensate the one-pole's low-frequency phase delay ≈ (1−a)/a samples
        // so the loop period stays on pitch as damping changes.
        let tune_comp = (1.0 - a) / a;
        let d = (period - tune_comp).clamp(2.0, MAX_DELAY);

        // Per-voice PRNG before the &mut self.pluck borrow.
        let white = self.white();

        let st = &mut *self.pluck;
        let y = st.delay.read_cubic(d);
        st.damp += a * (y - st.damp);

        // Sustain: loop gain compounds once per period, so audible decay goes
        // as g^freq per second — map harmonics exponentially toward 1.0 to
        // spread that usefully (0 = dead thud, 0.5 ≈ ½ s tail at A4, 1 = drone).
        let g = 1.0 - 0.2 * exp2f(-6.64 * harmonics);

        let excite = if st.burst_remaining > 0 {
            st.burst_remaining -= 1;
            let c = 0.05 + 0.95 * morph * morph;
            st.exciter_lp += c * (white - st.exciter_lp);
            st.exciter_lp
        } else {
            0.0
        };

        st.delay.write(excite + st.damp * g);
        st.dc.process(y) * 0.5
    }
}
