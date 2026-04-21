//! Band-limited oscillators with phase shaping.
//!
//! Provides a phasor-based oscillator system with multiple waveforms and
//! optional phase distortion. Anti-aliasing is achieved via PolyBLEP
//! (Polynomial Band-Limited Step) for discontinuous waveforms.
//!
//! # Waveforms
//!
//! | Name   | Alias | Description                              |
//! |--------|-------|------------------------------------------|
//! | `sine` | -     | Pure sinusoid                            |
//! | `tri`  | -     | Triangle wave                            |
//! | `saw`  | -     | Sawtooth with PolyBLEP anti-aliasing     |
//! | `zaw`  | -     | Raw sawtooth (no anti-aliasing)          |
//! | `pulse`| -     | Variable-width pulse with PolyBLEP       |
//! | `pulze`| -     | Raw pulse (no anti-aliasing)             |
//!
//! # Phase Shaping
//!
//! [`PhaseShape`] transforms the oscillator phase before waveform generation,
//! enabling complex timbres from simple waveforms:
//!
//! - **warp**: Power curve distortion shifts harmonic balance
//! - **mirror**: Reflection creates symmetrical waveforms
//! - **size**: Step quantization for lo-fi/bitcrushed effects

use super::fastmath::{exp2f, powf, sinf};
use crate::types::LfoShape;
use std::f32::consts::PI;

/// Wraps `phase + offset` into `[0, 1)`. Zero-alloc, handles any finite offset.
#[inline]
fn offset_phase(phase: f32, offset: f32) -> f32 {
    let p = phase + offset;
    p - p.floor()
}

/// PolyBLEP residual near a phase-wrap discontinuity.
///
/// Uses `|dt|` so the correction also fires for negative `dt` (e.g.
/// soft-sync's reversed direction).
pub(crate) fn poly_blep(t: f32, dt: f32) -> f32 {
    let adt = dt.abs();
    if t < adt {
        let t = t / adt;
        return t + t - t * t - 1.0;
    }
    if t > 1.0 - adt {
        let t = (t - 1.0) / adt;
        return t * t + t + t + 1.0;
    }
    0.0
}

// Two-sample polyBLEP/polyBLAMP lobes indexed by `wrap_frac = 1 − ν`
// (fraction of the sample period remaining after the event).

#[inline]
pub(crate) fn blep_pre_step(wrap_frac: f32) -> f32 {
    0.5 * wrap_frac * wrap_frac
}

#[inline]
pub(crate) fn blep_post_step(wrap_frac: f32) -> f32 {
    let d = 1.0 - wrap_frac;
    -0.5 * d * d
}

#[inline]
pub(crate) fn blamp_pre_kink(wrap_frac: f32) -> f32 {
    wrap_frac * wrap_frac * wrap_frac / 6.0
}

#[inline]
pub(crate) fn blamp_post_kink(wrap_frac: f32) -> f32 {
    let d = 1.0 - wrap_frac;
    d * d * d / 6.0
}

/// Band-limited square wave via PolyBLEP.
///
/// Returns `+1` while `phase < 0.5`, `-1` otherwise, with smoothed transitions
/// at both discontinuities (phase 0 and phase 0.5).
#[inline]
pub fn polyblep_square(phase: f32, dt: f32) -> f32 {
    let naive = if phase < 0.5 { 1.0 } else { -1.0 };
    let rise = poly_blep(phase, dt);
    let shifted = if phase >= 0.5 {
        phase - 0.5
    } else {
        phase + 0.5
    };
    let fall = poly_blep(shifted, dt);
    naive + rise - fall
}

/// Phase transformation parameters for waveform shaping.
///
/// Applies a chain of transformations to the oscillator phase:
/// warp → mirror → size (in that order).
///
/// All parameters have neutral defaults that result in no transformation.
#[derive(Clone, Copy)]
pub struct PhaseShape {
    /// Phase quantization steps. Values >= 2 create stair-step waveforms.
    pub size: u16,
    /// Power curve exponent. Positive values compress early phase,
    /// negative values compress late phase. Default: 0.0 (linear).
    pub warp: f32,
    /// Mirror/fold position in `[0, 1]`. Phase reflects at this point.
    /// Default: 0.0 (disabled).
    pub mirror: f32,
}

impl Default for PhaseShape {
    fn default() -> Self {
        Self {
            size: 0,
            warp: 0.0,
            mirror: 0.0,
        }
    }
}

impl PhaseShape {
    /// Returns `true` if any shaping parameter is non-neutral.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.size >= 2 || self.warp != 0.0 || self.mirror > 0.0
    }

    /// Applies the full transformation chain to a phase value.
    ///
    /// Input and output are in the range `[0, 1)`.
    /// Assumes `is_active()` returned true; call unconditionally for simplicity
    /// or guard with `is_active()` to skip the function call entirely.
    #[inline]
    pub fn apply(&self, phase: f32) -> f32 {
        let mut p = phase;

        // WARP: power curve asymmetry
        if self.warp != 0.0 {
            p = powf(p, exp2f(self.warp * 2.0));
        }

        // MIRROR: reflect at position
        if self.mirror > 0.0 && self.mirror < 1.0 {
            let m = self.mirror;
            p = if p < m {
                p / m
            } else {
                1.0 - (p - m) / (1.0 - m)
            };
        }

        // SIZE: quantize
        if self.size >= 2 {
            let steps = self.size as f32;
            p = ((p * steps).floor() / (steps - 1.0)).min(1.0);
        }

        p
    }
}

/// Phase accumulator with waveform generation methods.
///
/// Maintains a phase value in `[0, 1)` that advances each sample based on
/// frequency. Provides both stateful methods (advance phase) and stateless
/// methods (compute at arbitrary phase for unison/spread).
#[derive(Clone, Copy)]
pub struct Phasor {
    /// Current phase position in `[0, 1)`.
    pub phase: f32,
    /// Held value for sample-and-hold LFO.
    sh_value: f32,
    /// PRNG state for sample-and-hold randomization.
    sh_seed: u32,
}

impl Default for Phasor {
    fn default() -> Self {
        Self {
            phase: 0.0,
            sh_value: 0.0,
            sh_seed: 123456789,
        }
    }
}

impl Phasor {
    /// Advances the phase by one sample.
    ///
    /// - `freq`: Oscillator frequency in Hz
    /// - `isr`: Inverse sample rate (1.0 / sample_rate)
    pub fn update(&mut self, freq: f32, isr: f32) {
        self.phase += freq * isr;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        if self.phase < 0.0 {
            self.phase += 1.0;
        }
    }

    /// Generates an LFO sample for the given shape.
    ///
    /// Sample-and-hold (`Sh`) latches a new random value at each cycle start.
    pub fn lfo(&mut self, shape: LfoShape, freq: f32, isr: f32) -> f32 {
        self.lfo_pm(shape, freq, isr, 0.0)
    }

    /// Like `lfo` but reads from `(phase + phase_offset)` wrapped into `[0, 1)`.
    /// Used for phase-modulation feedback and nested FM where a modulator's
    /// phase is offset by another signal (in turns).
    ///
    /// `Sh` (sample-and-hold) ignores the offset: its output is latched at the
    /// true phase wrap and independent of read position.
    pub fn lfo_pm(&mut self, shape: LfoShape, freq: f32, isr: f32, phase_offset: f32) -> f32 {
        let p_true = self.phase;
        let p = offset_phase(p_true, phase_offset);
        self.update(freq, isr);

        match shape {
            LfoShape::Sine => sinf(p * 2.0 * PI),
            LfoShape::Tri => {
                if p < 0.5 {
                    4.0 * p - 1.0
                } else {
                    3.0 - 4.0 * p
                }
            }
            LfoShape::Saw => p * 2.0 - 1.0,
            LfoShape::Square => {
                if p < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            LfoShape::Sh => {
                if self.phase < p_true {
                    self.sh_seed = self.sh_seed.wrapping_mul(1103515245).wrapping_add(12345);
                    self.sh_value = ((self.sh_seed >> 16) & 0x7fff) as f32 / 16383.5 - 1.0;
                }
                self.sh_value
            }
        }
    }

    /// Pure sine wave.
    pub fn sine(&mut self, freq: f32, isr: f32) -> f32 {
        let s = sinf(self.phase * 2.0 * PI);
        self.update(freq, isr);
        s
    }

    /// Triangle wave (no anti-aliasing needed, naturally band-limited).
    pub fn tri(&mut self, freq: f32, isr: f32) -> f32 {
        let s = if self.phase < 0.5 {
            4.0 * self.phase - 1.0
        } else {
            3.0 - 4.0 * self.phase
        };
        self.update(freq, isr);
        s
    }

    /// Band-limited sawtooth using PolyBLEP.
    pub fn saw(&mut self, freq: f32, isr: f32) -> f32 {
        let dt = freq * isr;
        let p = poly_blep(self.phase, dt);
        let s = self.phase * 2.0 - 1.0 - p;
        self.update(freq, isr);
        s
    }

    /// Raw sawtooth without anti-aliasing.
    ///
    /// Use for low frequencies or when aliasing is acceptable/desired.
    pub fn zaw(&mut self, freq: f32, isr: f32) -> f32 {
        let s = self.phase * 2.0 - 1.0;
        self.update(freq, isr);
        s
    }

    /// Band-limited pulse wave with variable width using PolyBLEP.
    ///
    /// - `pw`: Pulse width in `[0, 1]`. 0.5 = square wave.
    pub fn pulse(&mut self, freq: f32, pw: f32, isr: f32) -> f32 {
        let dt = freq * isr;
        let mut phi = self.phase + pw;
        if phi >= 1.0 {
            phi -= 1.0;
        }
        let p1 = poly_blep(phi, dt);
        let p2 = poly_blep(self.phase, dt);
        let pulse = 2.0 * (self.phase - phi) - p2 + p1;
        self.update(freq, isr);
        pulse + pw * 2.0 - 1.0
    }

    /// Raw pulse wave without anti-aliasing.
    ///
    /// - `duty`: Duty cycle in `[0, 1]`. 0.5 = square wave.
    pub fn pulze(&mut self, freq: f32, duty: f32, isr: f32) -> f32 {
        let s = if self.phase < duty { 1.0 } else { -1.0 };
        self.update(freq, isr);
        s
    }

    /// Sine wave with phase shaping and optional phase-modulation offset (turns).
    pub fn sine_shaped(
        &mut self,
        freq: f32,
        isr: f32,
        shape: &PhaseShape,
        phase_offset: f32,
    ) -> f32 {
        let read = offset_phase(self.phase, phase_offset);
        let p = if shape.is_active() {
            shape.apply(read)
        } else {
            read
        };
        let s = sinf(p * 2.0 * PI);
        self.update(freq, isr);
        s
    }

    /// Triangle wave with phase shaping and optional PM offset (turns).
    pub fn tri_shaped(
        &mut self,
        freq: f32,
        isr: f32,
        shape: &PhaseShape,
        phase_offset: f32,
    ) -> f32 {
        let read = offset_phase(self.phase, phase_offset);
        let p = if shape.is_active() {
            shape.apply(read)
        } else {
            read
        };
        let s = if p < 0.5 {
            4.0 * p - 1.0
        } else {
            3.0 - 4.0 * p
        };
        self.update(freq, isr);
        s
    }

    /// Sawtooth with phase shaping, PolyBLEP anti-aliasing, optional PM offset (turns).
    pub fn saw_shaped(
        &mut self,
        freq: f32,
        isr: f32,
        shape: &PhaseShape,
        phase_offset: f32,
    ) -> f32 {
        let dt = freq * isr;
        let read = offset_phase(self.phase, phase_offset);
        let p = if shape.is_active() {
            shape.apply(read)
        } else {
            read
        };
        let blep = poly_blep(p, dt);
        let s = p * 2.0 - 1.0 - blep;
        self.update(freq, isr);
        s
    }

    /// Raw sawtooth with phase shaping and optional PM offset (turns).
    pub fn zaw_shaped(
        &mut self,
        freq: f32,
        isr: f32,
        shape: &PhaseShape,
        phase_offset: f32,
    ) -> f32 {
        let read = offset_phase(self.phase, phase_offset);
        let p = if shape.is_active() {
            shape.apply(read)
        } else {
            read
        };
        let s = p * 2.0 - 1.0;
        self.update(freq, isr);
        s
    }

    /// Pulse wave with phase shaping, PolyBLEP anti-aliasing, optional PM offset (turns).
    pub fn pulse_shaped(
        &mut self,
        freq: f32,
        pw: f32,
        isr: f32,
        shape: &PhaseShape,
        phase_offset: f32,
    ) -> f32 {
        let dt = freq * isr;
        let read = offset_phase(self.phase, phase_offset);
        let p = if shape.is_active() {
            shape.apply(read)
        } else {
            read
        };
        let mut phi = p + pw;
        if phi >= 1.0 {
            phi -= 1.0;
        }
        let p1 = poly_blep(phi, dt);
        let p2 = poly_blep(p, dt);
        let s = 2.0 * (p - phi) - p2 + p1 + pw * 2.0 - 1.0;
        self.update(freq, isr);
        s
    }

    /// Raw pulse with phase shaping and optional PM offset (turns).
    pub fn pulze_shaped(
        &mut self,
        freq: f32,
        duty: f32,
        isr: f32,
        shape: &PhaseShape,
        phase_offset: f32,
    ) -> f32 {
        let read = offset_phase(self.phase, phase_offset);
        let p = if shape.is_active() {
            shape.apply(read)
        } else {
            read
        };
        let s = if p < duty { 1.0 } else { -1.0 };
        self.update(freq, isr);
        s
    }

    // -------------------------------------------------------------------------
    // Stateless variants for unison/spread - compute at arbitrary phase
    // -------------------------------------------------------------------------

    /// Sine at arbitrary phase (stateless, for unison voices).
    #[inline]
    pub fn sine_at(phase: f32, shape: &PhaseShape) -> f32 {
        let p = if shape.is_active() {
            shape.apply(phase)
        } else {
            phase
        };
        sinf(p * 2.0 * PI)
    }

    /// Triangle at arbitrary phase (stateless, for unison voices).
    #[inline]
    pub fn tri_at(phase: f32, shape: &PhaseShape) -> f32 {
        let p = if shape.is_active() {
            shape.apply(phase)
        } else {
            phase
        };
        if p < 0.5 {
            4.0 * p - 1.0
        } else {
            3.0 - 4.0 * p
        }
    }

    /// Band-limited sawtooth at arbitrary phase (stateless, for unison voices).
    #[inline]
    pub fn saw_at(phase: f32, dt: f32, shape: &PhaseShape) -> f32 {
        let p = if shape.is_active() {
            shape.apply(phase)
        } else {
            phase
        };
        let blep = poly_blep(p, dt);
        p * 2.0 - 1.0 - blep
    }

    /// Raw sawtooth at arbitrary phase (stateless, for unison voices).
    #[inline]
    pub fn zaw_at(phase: f32, shape: &PhaseShape) -> f32 {
        let p = if shape.is_active() {
            shape.apply(phase)
        } else {
            phase
        };
        p * 2.0 - 1.0
    }

    /// Band-limited pulse at arbitrary phase (stateless, for unison voices).
    #[inline]
    pub fn pulse_at(phase: f32, dt: f32, pw: f32, shape: &PhaseShape) -> f32 {
        let p = if shape.is_active() {
            shape.apply(phase)
        } else {
            phase
        };
        let mut phi = p + pw;
        if phi >= 1.0 {
            phi -= 1.0;
        }
        let p1 = poly_blep(phi, dt);
        let p2 = poly_blep(p, dt);
        2.0 * (p - phi) - p2 + p1 + pw * 2.0 - 1.0
    }

    /// Raw pulse at arbitrary phase (stateless, for unison voices).
    #[inline]
    pub fn pulze_at(phase: f32, duty: f32, shape: &PhaseShape) -> f32 {
        let p = if shape.is_active() {
            shape.apply(phase)
        } else {
            phase
        };
        if p < duty {
            1.0
        } else {
            -1.0
        }
    }
}
