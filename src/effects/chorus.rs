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

use crate::dsp::Phasor;

/// Delay buffer size in samples (~42ms at 48kHz).
const BUFFER_SIZE: usize = 2048;

/// Number of chorus voices (phase-offset delay taps).
const VOICES: usize = 3;

/// Multi-voice stereo chorus effect.
///
/// Uses a circular delay buffer with three LFO-modulated tap points.
/// The LFOs are phase-offset by 1/3 cycle (120°) to create smooth,
/// non-pulsing modulation.
#[derive(Clone, Copy)]
pub struct Chorus {
    /// Circular delay buffer (mono, power-of-2 for efficient wrapping).
    buffer: [f32; BUFFER_SIZE],
    /// Current write position in the delay buffer.
    write_pos: usize,
    /// Per-voice LFOs for delay time modulation.
    lfo: [Phasor; VOICES],
}

impl Default for Chorus {
    fn default() -> Self {
        let mut lfo = [Phasor::default(); VOICES];
        // Distribute LFO phases evenly: 0°, 120°, 240°
        for (i, l) in lfo.iter_mut().enumerate() {
            l.phase = i as f32 / VOICES as f32;
        }
        Self {
            buffer: [0.0; BUFFER_SIZE],
            write_pos: 0,
            lfo,
        }
    }
}

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
        let mod_range = delay_ms * 0.8;

        // Sum to mono for delay line (common chorus technique)
        let mono = (left + right) * 0.5;
        self.buffer[self.write_pos] = mono;

        let mut out_l = 0.0_f32;
        let mut out_r = 0.0_f32;

        let min_delay = 1.5;
        let max_delay = 50.0_f32.min((BUFFER_SIZE as f32 - 2.0) * 1000.0 / sr);

        for v in 0..VOICES {
            let lfo = self.lfo[v].sine(rate, isr);

            // Opposite modulation for L/R creates stereo width
            let modulation = depth * mod_range * lfo;
            let dly_l = (delay_ms + modulation).clamp(min_delay, max_delay);
            let dly_r = (delay_ms - modulation).clamp(min_delay, max_delay);

            // Convert ms to samples
            let samp_l = (dly_l * sr * 0.001).clamp(1.0, BUFFER_SIZE as f32 - 2.0);
            let samp_r = (dly_r * sr * 0.001).clamp(1.0, BUFFER_SIZE as f32 - 2.0);

            // Linear interpolation for sub-sample accuracy
            let pos_l = samp_l.floor() as usize;
            let frac_l = samp_l - pos_l as f32;
            let idx_l0 = (self.write_pos + BUFFER_SIZE - pos_l) & (BUFFER_SIZE - 1);
            let idx_l1 = (self.write_pos + BUFFER_SIZE - pos_l - 1) & (BUFFER_SIZE - 1);
            let tap_l = self.buffer[idx_l0] + frac_l * (self.buffer[idx_l1] - self.buffer[idx_l0]);

            let pos_r = samp_r.floor() as usize;
            let frac_r = samp_r - pos_r as f32;
            let idx_r0 = (self.write_pos + BUFFER_SIZE - pos_r) & (BUFFER_SIZE - 1);
            let idx_r1 = (self.write_pos + BUFFER_SIZE - pos_r - 1) & (BUFFER_SIZE - 1);
            let tap_r = self.buffer[idx_r0] + frac_r * (self.buffer[idx_r1] - self.buffer[idx_r0]);

            out_l += tap_l;
            out_r += tap_r;
        }

        self.write_pos = (self.write_pos + 1) & (BUFFER_SIZE - 1);

        // Average the voices
        out_l /= VOICES as f32;
        out_r /= VOICES as f32;

        // Equal-power mix: dry × 0.707 + wet × 0.707
        const MIX: f32 = std::f32::consts::FRAC_1_SQRT_2;
        [mono * MIX + out_l * MIX, mono * MIX + out_r * MIX]
    }
}
