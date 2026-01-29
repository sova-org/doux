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
//! - **mult**: Phase multiplication creates harmonic partials
//! - **warp**: Power curve distortion shifts harmonic balance
//! - **mirror**: Reflection creates symmetrical waveforms
//! - **size**: Step quantization for lo-fi/bitcrushed effects

use super::fastmath::{exp2f, powf, sinf};
use crate::types::LfoShape;
use std::f32::consts::PI;

/// PolyBLEP correction for band-limited discontinuities.
///
/// Applies a polynomial correction near waveform discontinuities to reduce
/// aliasing. The correction is applied within one sample of the transition.
///
/// - `t`: Current phase position in `[0, 1)`
/// - `dt`: Phase increment per sample (frequency × inverse sample rate)
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        return t + t - t * t - 1.0;
    }
    if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        return t * t + t + t + 1.0;
    }
    0.0
}

/// Phase transformation parameters for waveform shaping.
///
/// Applies a chain of transformations to the oscillator phase:
/// mult → warp → mirror → size (in that order).
///
/// All parameters have neutral defaults that result in no transformation.
#[derive(Clone, Copy)]
pub struct PhaseShape {
    /// Phase multiplier. Values > 1 create harmonic overtones.
    pub size: u16,
    /// Phase multiplication factor. Default: 1.0 (no multiplication).
    pub mult: f32,
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
            mult: 1.0,
            warp: 0.0,
            mirror: 0.0,
        }
    }
}

impl PhaseShape {
    /// Returns `true` if any shaping parameter is non-neutral.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.size >= 2 || self.mult != 1.0 || self.warp != 0.0 || self.mirror > 0.0
    }

    /// Returns the effective phase rate multiplier for PolyBLEP scaling.
    ///
    /// The shaped phase traverses faster when mult > 1, so the PolyBLEP
    /// correction window must widen accordingly.
    #[inline]
    pub fn effective_mult(&self) -> f32 {
        if self.mult > 1.0 {
            self.mult
        } else {
            1.0
        }
    }

    /// Applies the full transformation chain to a phase value.
    ///
    /// Input and output are in the range `[0, 1)`.
    /// Assumes `is_active()` returned true; call unconditionally for simplicity
    /// or guard with `is_active()` to skip the function call entirely.
    #[inline]
    pub fn apply(&self, phase: f32) -> f32 {
        let mut p = phase;

        // MULT: multiply and wrap
        if self.mult != 1.0 {
            p = (p * self.mult).fract();
            if p < 0.0 {
                p += 1.0;
            }
        }

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
        let p = self.phase;
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
                if self.phase < p {
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

    /// Sine wave with phase shaping.
    pub fn sine_shaped(&mut self, freq: f32, isr: f32, shape: &PhaseShape) -> f32 {
        let p = if shape.is_active() {
            shape.apply(self.phase)
        } else {
            self.phase
        };
        let s = sinf(p * 2.0 * PI);
        self.update(freq, isr);
        s
    }

    /// Triangle wave with phase shaping.
    pub fn tri_shaped(&mut self, freq: f32, isr: f32, shape: &PhaseShape) -> f32 {
        let p = if shape.is_active() {
            shape.apply(self.phase)
        } else {
            self.phase
        };
        let s = if p < 0.5 {
            4.0 * p - 1.0
        } else {
            3.0 - 4.0 * p
        };
        self.update(freq, isr);
        s
    }

    /// Sawtooth with phase shaping and PolyBLEP anti-aliasing.
    pub fn saw_shaped(&mut self, freq: f32, isr: f32, shape: &PhaseShape) -> f32 {
        if !shape.is_active() {
            return self.saw(freq, isr);
        }
        let dt = freq * isr;
        let p = shape.apply(self.phase);
        let blep = poly_blep(p, dt * shape.effective_mult());
        let s = p * 2.0 - 1.0 - blep;
        self.update(freq, isr);
        s
    }

    /// Raw sawtooth with phase shaping.
    pub fn zaw_shaped(&mut self, freq: f32, isr: f32, shape: &PhaseShape) -> f32 {
        let p = if shape.is_active() {
            shape.apply(self.phase)
        } else {
            self.phase
        };
        let s = p * 2.0 - 1.0;
        self.update(freq, isr);
        s
    }

    /// Pulse wave with phase shaping and PolyBLEP anti-aliasing.
    pub fn pulse_shaped(&mut self, freq: f32, pw: f32, isr: f32, shape: &PhaseShape) -> f32 {
        if !shape.is_active() {
            return self.pulse(freq, pw, isr);
        }
        let dt = freq * isr * shape.effective_mult();
        let p = shape.apply(self.phase);
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

    /// Raw pulse with phase shaping.
    pub fn pulze_shaped(&mut self, freq: f32, duty: f32, isr: f32, shape: &PhaseShape) -> f32 {
        let p = if shape.is_active() {
            shape.apply(self.phase)
        } else {
            self.phase
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
        let blep = poly_blep(p, dt * shape.effective_mult());
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
        let dt_eff = dt * shape.effective_mult();
        let mut phi = p + pw;
        if phi >= 1.0 {
            phi -= 1.0;
        }
        let p1 = poly_blep(phi, dt_eff);
        let p2 = poly_blep(p, dt_eff);
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
