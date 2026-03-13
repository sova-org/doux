//! DAHDSR envelope generation for audio synthesis.
//!
//! This module provides a state-machine based DAHDSR (Delay, Attack, Hold, Decay, Sustain, Release)
//! envelope generator with configurable curve shapes. The envelope is self-timed via a gate
//! duration and produces amplitude values in the range `[0.0, 1.0]`.
//!
//! # Phase Order
//!
//! 1. Delay — output 0.0 for `delay` seconds
//! 2. Attack — rise from 0.0 to 1.0
//! 3. Hold — stay at 1.0 for `hold` seconds
//! 4. Decay — fall from 1.0 to sustain level
//! 5. Sustain — hold at sustain level until gate time elapses
//! 6. Release — fall to 0.0
//!
//! # Gate Duration
//!
//! `gate` = delay + attack + hold + decay + sustain_time.
//! If gate = 0, sustain is infinite (requires explicit release).
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

/// Current phase of the DAHDSR envelope state machine.
#[derive(Clone, Copy)]
pub enum DahdsrState {
    /// Envelope is inactive, outputting zero.
    Off,
    /// Waiting before attack begins.
    Delay,
    /// Rising from current value toward peak (1.0).
    Attack,
    /// Holding at peak (1.0) for a fixed duration.
    Hold,
    /// Falling from peak toward sustain level.
    Decay,
    /// Holding at sustain level while gate time remains.
    Sustain,
    /// Falling from current value toward zero after gate release.
    Release,
}

/// State-machine DAHDSR envelope generator.
///
/// Self-timed: call [`Dahdsr::trigger`] to start, then [`Dahdsr::update`] each sample.
/// The gate duration controls when release starts automatically.
///
/// # Curve Parameters
///
/// Default curves use an exponent of `2.0` for attack (convex) and decay/release
/// (concave when negated internally), producing natural-sounding amplitude shapes.
#[derive(Clone, Copy)]
pub struct Dahdsr {
    state: DahdsrState,
    phase_time: f32,
    elapsed: f32,
    start_val: f32,
    current_val: f32,
    gate_time: f32,
    attack_curve: f32,
    decay_curve: f32,
}

impl Default for Dahdsr {
    fn default() -> Self {
        Self {
            state: DahdsrState::Off,
            phase_time: 0.0,
            elapsed: 0.0,
            start_val: 0.0,
            current_val: 0.0,
            gate_time: 0.0,
            attack_curve: 2.0,
            decay_curve: 2.0,
        }
    }
}

impl Dahdsr {
    /// Returns `true` if the envelope is in the [`DahdsrState::Off`] state.
    pub fn is_off(&self) -> bool {
        matches!(self.state, DahdsrState::Off)
    }

    /// Start the envelope. `gate` is the total time before release (0.0 = infinite).
    pub fn trigger(&mut self, gate: f32) {
        self.gate_time = gate;
        self.elapsed = 0.0;
        self.phase_time = 0.0;
        self.start_val = self.current_val;
        self.state = DahdsrState::Delay;
    }

    /// Transition to Release from any active phase.
    pub fn force_release(&mut self) {
        if matches!(self.state, DahdsrState::Off | DahdsrState::Release) {
            return;
        }
        self.state = DahdsrState::Release;
        self.start_val = self.current_val;
        self.phase_time = 0.0;
    }

    /// Check if gate time has elapsed and auto-release if so.
    #[inline]
    fn check_gate(&mut self) -> bool {
        if self.gate_time > 0.0 && self.elapsed >= self.gate_time {
            self.state = DahdsrState::Release;
            self.start_val = self.current_val;
            self.phase_time = 0.0;
            true
        } else {
            false
        }
    }

    /// Advances the envelope state machine and returns the current amplitude.
    ///
    /// # Parameters
    ///
    /// - `isr`: Inverse sample rate (1.0 / sample_rate)
    /// - `delay`: Delay duration in seconds
    /// - `attack`: Attack duration in seconds
    /// - `hold`: Hold duration in seconds
    /// - `decay`: Decay duration in seconds
    /// - `sustain`: Sustain level in range `[0.0, 1.0]`
    /// - `release`: Release duration in seconds
    pub fn update(
        &mut self,
        isr: f32,
        delay: f32,
        attack: f32,
        hold: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> f32 {
        match self.state {
            DahdsrState::Off => {
                self.current_val = 0.0;
                0.0
            }
            DahdsrState::Delay => {
                self.phase_time += isr;
                self.elapsed += isr;
                if self.check_gate() {
                    return self.current_val;
                }
                if delay <= 0.0 || self.phase_time >= delay {
                    self.state = DahdsrState::Attack;
                    self.phase_time = 0.0;
                }
                self.current_val = lerp(0.0, self.start_val, 0.0, 0.0);
                // During delay, fade from retrigger value toward 0
                if self.start_val > 0.0 {
                    let t = if delay > 0.0 { self.phase_time / delay } else { 1.0 };
                    self.current_val = lerp(t, self.start_val, 0.0, 0.0);
                } else {
                    self.current_val = 0.0;
                }
                self.current_val
            }
            DahdsrState::Attack => {
                self.phase_time += isr;
                self.elapsed += isr;
                if self.check_gate() {
                    return self.current_val;
                }
                let val = lerp(self.phase_time / attack, self.start_val, 1.0, self.attack_curve);
                if val > 0.9999 {
                    self.state = DahdsrState::Hold;
                    self.phase_time = 0.0;
                    self.current_val = 1.0;
                    return 1.0;
                }
                self.current_val = val;
                val
            }
            DahdsrState::Hold => {
                self.phase_time += isr;
                self.elapsed += isr;
                if self.check_gate() {
                    return self.current_val;
                }
                if hold <= 0.0 || self.phase_time >= hold {
                    self.state = DahdsrState::Decay;
                    self.phase_time = 0.0;
                }
                self.current_val = 1.0;
                1.0
            }
            DahdsrState::Decay => {
                self.phase_time += isr;
                self.elapsed += isr;
                if self.check_gate() {
                    return self.current_val;
                }
                let val = lerp(self.phase_time / decay, 1.0, sustain, -self.decay_curve);
                if (val - sustain).abs() < 0.0001 {
                    self.state = DahdsrState::Sustain;
                    self.phase_time = 0.0;
                    self.current_val = sustain;
                    return sustain;
                }
                self.current_val = val;
                val
            }
            DahdsrState::Sustain => {
                self.phase_time += isr;
                self.elapsed += isr;
                if self.check_gate() {
                    return self.current_val;
                }
                self.current_val = sustain;
                sustain
            }
            DahdsrState::Release => {
                self.phase_time += isr;
                let val = lerp(self.phase_time / release, self.start_val, 0.0, -self.decay_curve);
                if val < 0.0001 {
                    self.state = DahdsrState::Off;
                    self.current_val = 0.0;
                    return 0.0;
                }
                self.current_val = val;
                val
            }
        }
    }
}

/// Parsed envelope parameters with activation flag.
#[derive(Clone, Copy, Default)]
pub struct EnvelopeParams {
    /// Overall envelope amplitude multiplier.
    pub env: f32,
    /// Delay time in seconds.
    pub dly: f32,
    /// Attack time in seconds.
    pub att: f32,
    /// Hold time in seconds.
    pub hld: f32,
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
pub fn init_envelope(
    env: Option<f32>,
    dly: Option<f32>,
    att: Option<f32>,
    hld: Option<f32>,
    dec: Option<f32>,
    sus: Option<f32>,
    rel: Option<f32>,
) -> EnvelopeParams {
    if env.is_none() && dly.is_none() && att.is_none() && hld.is_none()
        && dec.is_none() && sus.is_none() && rel.is_none()
    {
        return EnvelopeParams {
            env: 1.0,
            dly: 0.0,
            att: 0.003,
            hld: 0.0,
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
        dly: dly.unwrap_or(0.0),
        att: att.unwrap_or(0.003),
        hld: hld.unwrap_or(0.0),
        dec: dec.unwrap_or(0.0),
        sus: sus_val,
        rel: rel.unwrap_or(0.005),
        active: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn max_discontinuity(samples: &[f32]) -> f32 {
        samples
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0, f32::max)
    }

    #[test]
    fn all_transitions_smooth() {
        let mut env = Dahdsr::default();
        let isr = 1.0 / 44100.0;
        let mut samples = Vec::new();

        // Trigger with gate=0 (infinite sustain)
        env.trigger(0.0);

        // Full DAHDSR cycle
        for _ in 0..10000 {
            let val = env.update(isr, 0.0, 0.01, 0.0, 0.05, 0.5, 0.01);
            samples.push(val);
        }

        // Force release
        env.force_release();
        while !env.is_off() {
            let val = env.update(isr, 0.0, 0.01, 0.0, 0.05, 0.5, 0.01);
            samples.push(val);
        }

        let max_delta = max_discontinuity(&samples);
        assert!(max_delta < 0.01, "Discontinuity: max delta = {}", max_delta);
    }

    #[test]
    fn short_attack_no_click() {
        let mut env = Dahdsr::default();
        let isr = 1.0 / 44100.0;
        let mut samples = Vec::new();

        env.trigger(0.0);

        for _ in 0..500 {
            let val = env.update(isr, 0.0, 0.001, 0.0, 0.0, 1.0, 0.01);
            samples.push(val);
        }

        let max_delta = max_discontinuity(&samples);
        assert!(
            max_delta < 0.05,
            "Attack discontinuity: max delta = {}",
            max_delta
        );
    }

    #[test]
    fn short_release_no_click() {
        let mut env = Dahdsr::default();
        let isr = 1.0 / 44100.0;
        let mut samples = Vec::new();

        env.trigger(0.0);

        // Attack to sustain
        for _ in 0..500 {
            env.update(isr, 0.0, 0.01, 0.0, 0.0, 1.0, 0.001);
        }

        // Force release
        env.force_release();
        while !env.is_off() {
            let val = env.update(isr, 0.0, 0.01, 0.0, 0.0, 1.0, 0.001);
            samples.push(val);
        }

        let max_delta = max_discontinuity(&samples);
        assert!(
            max_delta < 0.05,
            "Release discontinuity: max delta = {}",
            max_delta
        );
    }

    #[test]
    fn gate_auto_release() {
        let mut env = Dahdsr::default();
        let isr = 1.0 / 44100.0;

        // gate = 0.1s, attack=0.01, no hold, no decay
        env.trigger(0.1);

        let mut last_val = 0.0;
        let mut reached_release = false;
        for _ in 0..10000 {
            let val = env.update(isr, 0.0, 0.01, 0.0, 0.0, 1.0, 0.01);
            if matches!(env.state, DahdsrState::Release) {
                reached_release = true;
            }
            if env.is_off() {
                break;
            }
            last_val = val;
        }
        assert!(reached_release, "Should have auto-released");
        assert!(last_val < 0.01 || env.is_off(), "Should have faded out");
    }

    #[test]
    fn delay_phase() {
        let mut env = Dahdsr::default();
        let isr = 1.0 / 44100.0;

        env.trigger(0.0);

        // With 0.1s delay, first samples should be 0
        let val = env.update(isr, 0.1, 0.01, 0.0, 0.0, 1.0, 0.01);
        assert!(val.abs() < 0.001, "Should be near zero during delay");
        assert!(matches!(env.state, DahdsrState::Delay));
    }

    #[test]
    fn hold_phase() {
        let mut env = Dahdsr::default();
        let isr = 1.0 / 44100.0;

        env.trigger(0.0);

        // Skip through attack (very short)
        for _ in 0..500 {
            env.update(isr, 0.0, 0.001, 0.05, 0.05, 0.5, 0.01);
        }

        // Should be in hold, outputting 1.0
        assert!(matches!(env.state, DahdsrState::Hold) || matches!(env.state, DahdsrState::Decay));
    }

    #[test]
    fn infinite_gate_stays_sustaining() {
        let mut env = Dahdsr::default();
        let isr = 1.0 / 44100.0;

        env.trigger(0.0); // infinite

        // Run through attack+decay
        for _ in 0..44100 {
            env.update(isr, 0.0, 0.01, 0.0, 0.05, 0.5, 0.01);
        }

        // Should still be sustaining, not released
        assert!(
            !env.is_off(),
            "gate=0 should mean infinite sustain"
        );
        assert!(
            matches!(env.state, DahdsrState::Sustain),
            "Should be in Sustain state"
        );
    }
}
