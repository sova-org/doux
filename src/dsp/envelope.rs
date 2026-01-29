//! ADSR envelope generation for audio synthesis.
//!
//! This module provides a state-machine based ADSR (Attack, Decay, Sustain, Release)
//! envelope generator with configurable curve shapes. The envelope responds to gate
//! signals and produces amplitude values in the range `[0.0, 1.0]`.
//!
//! # Curve Shaping
//!
//! Attack and decay/release phases use exponential curves controlled by internal
//! parameters. Positive exponents create convex curves (slow start, fast finish),
//! while negative exponents create concave curves (fast start, slow finish).

use super::fastmath::powf;

/// Attempt to scale the input `x` from range `[0, 1]` to range `[y0, y1]` with an exponent `exp`.
///
/// Attempt because the expression `powf(1.0 - x, -exp)` can lead to a NaN when `exp` is greater than 1.0.
/// Using this function on 1.0 - x reverses the curve direction.
///
/// - `exp > 0`: Convex curve (slow start, accelerates toward end)
/// - `exp < 0`: Concave curve (fast start, decelerates toward end)
/// - `exp == 0`: Linear interpolation
fn lerp(x: f32, y0: f32, y1: f32, exp: f32) -> f32 {
    if x <= 0.0 {
        return y0;
    }
    if x >= 1.0 {
        return y1;
    }
    let curved = if exp == 0.0 {
        x
    } else if exp > 0.0 {
        powf(x, exp)
    } else {
        1.0 - powf(1.0 - x, -exp)
    };
    y0 + (y1 - y0) * curved
}

/// Current phase of the ADSR envelope state machine.
#[derive(Clone, Copy)]
pub enum AdsrState {
    /// Envelope is inactive, outputting zero.
    Off,
    /// Rising from current value toward peak (1.0).
    Attack,
    /// Falling from peak toward sustain level.
    Decay,
    /// Holding at sustain level while gate remains high.
    Sustain,
    /// Falling from current value toward zero after gate release.
    Release,
}

/// State-machine ADSR envelope generator.
///
/// Tracks envelope phase and timing internally. Call [`Adsr::update`] each sample
/// with the current time and gate signal to produce envelope values.
///
/// # Curve Parameters
///
/// Default curves use an exponent of `2.0` for attack (convex) and decay/release
/// (concave when negated internally), producing natural-sounding amplitude shapes.
#[derive(Clone, Copy)]
pub struct Adsr {
    state: AdsrState,
    start_time: f32,
    start_val: f32,
    attack_curve: f32,
    decay_curve: f32,
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            state: AdsrState::Off,
            start_time: 0.0,
            start_val: 0.0,
            attack_curve: 2.0,
            decay_curve: 2.0,
        }
    }
}

impl Adsr {
    /// Returns `true` if the envelope is in the [`AdsrState::Off`] state.
    pub fn is_off(&self) -> bool {
        matches!(self.state, AdsrState::Off)
    }

    /// Advances the envelope state machine and returns the current amplitude.
    ///
    /// The envelope responds to gate transitions:
    /// - Gate going high (`> 0.0`) triggers attack from current value
    /// - Gate going low (`<= 0.0`) triggers release from current value
    ///
    /// This allows retriggering during any phase without clicks, as the envelope
    /// always starts from its current position rather than jumping to zero.
    ///
    /// # Parameters
    ///
    /// - `time`: Current time in seconds (must be monotonically increasing)
    /// - `gate`: Gate signal (`> 0.0` = note on, `<= 0.0` = note off)
    /// - `attack`: Attack duration in seconds
    /// - `decay`: Decay duration in seconds
    /// - `sustain`: Sustain level in range `[0.0, 1.0]`
    /// - `release`: Release duration in seconds
    ///
    /// # Returns
    ///
    /// Envelope amplitude in range `[0.0, 1.0]`.
    pub fn update(
        &mut self,
        time: f32,
        gate: f32,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> f32 {
        match self.state {
            AdsrState::Off => {
                if gate > 0.0 {
                    self.state = AdsrState::Attack;
                    self.start_time = time;
                    self.start_val = 0.0;
                }
                0.0
            }
            AdsrState::Attack => {
                let t = time - self.start_time;
                if t > attack {
                    self.state = AdsrState::Decay;
                    self.start_time = time;
                    return 1.0;
                }
                lerp(t / attack, self.start_val, 1.0, self.attack_curve)
            }
            AdsrState::Decay => {
                let t = time - self.start_time;
                let val = lerp(t / decay, 1.0, sustain, -self.decay_curve);
                if gate <= 0.0 {
                    self.state = AdsrState::Release;
                    self.start_time = time;
                    self.start_val = val;
                    return val;
                }
                if t > decay {
                    self.state = AdsrState::Sustain;
                    self.start_time = time;
                    return sustain;
                }
                val
            }
            AdsrState::Sustain => {
                if gate <= 0.0 {
                    self.state = AdsrState::Release;
                    self.start_time = time;
                    self.start_val = sustain;
                }
                sustain
            }
            AdsrState::Release => {
                let t = time - self.start_time;
                if t > release {
                    self.state = AdsrState::Off;
                    return 0.0;
                }
                let val = lerp(t / release, self.start_val, 0.0, -self.decay_curve);
                if gate > 0.0 {
                    self.state = AdsrState::Attack;
                    self.start_time = time;
                    self.start_val = val;
                }
                val
            }
        }
    }
}

/// Parsed envelope parameters with activation flag.
///
/// Used to pass envelope configuration from pattern parsing to voice rendering.
/// The `active` field indicates whether the user explicitly specified any
/// envelope parameters, allowing voices to skip envelope processing when unused.
#[derive(Clone, Copy, Default)]
pub struct EnvelopeParams {
    /// Overall envelope amplitude multiplier.
    pub env: f32,
    /// Attack time in seconds.
    pub att: f32,
    /// Decay time in seconds.
    pub dec: f32,
    /// Sustain level in range `[0.0, 1.0]`.
    pub sus: f32,
    /// Release time in seconds.
    pub rel: f32,
    /// Whether envelope parameters were explicitly provided.
    pub active: bool,
}

/// Constructs envelope parameters from optional user inputs.
///
/// Applies sensible defaults and infers sustain level from context:
/// - If sustain is explicit, use it (clamped to `1.0`)
/// - If only attack is set, sustain defaults to `1.0` (full level after attack)
/// - If decay is set (with or without attack), sustain defaults to `0.0`
/// - Otherwise, sustain defaults to `1.0`
///
/// When no parameters are provided, returns inactive defaults suitable for
/// bypassing envelope processing entirely.
///
/// # Default Values
///
/// | Parameter | Default |
/// |-----------|---------|
/// | `env`     | `1.0`   |
/// | `att`     | `0.001` |
/// | `dec`     | `0.0`   |
/// | `sus`     | `1.0`   |
/// | `rel`     | `0.005` |
pub fn init_envelope(
    env: Option<f32>,
    att: Option<f32>,
    dec: Option<f32>,
    sus: Option<f32>,
    rel: Option<f32>,
) -> EnvelopeParams {
    if env.is_none() && att.is_none() && dec.is_none() && sus.is_none() && rel.is_none() {
        return EnvelopeParams {
            env: 1.0,
            att: 0.003,
            dec: 0.0,
            sus: 1.0,
            rel: 0.005,
            active: false,
        };
    }

    let sus_val = match (sus, att, dec) {
        (Some(s), _, _) => s.min(1.0),
        (None, Some(_), None) => 1.0,
        (None, None, Some(_)) => 0.0,
        (None, Some(_), Some(_)) => 0.0,
        _ => 1.0,
    };

    EnvelopeParams {
        env: env.unwrap_or(1.0),
        att: att.unwrap_or(0.003),
        dec: dec.unwrap_or(0.0),
        sus: sus_val,
        rel: rel.unwrap_or(0.005),
        active: true,
    }
}
