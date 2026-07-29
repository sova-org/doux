//! Modal resonator: eight tuned bandpasses standing in for the modes of a struck body.
//! Whatever is fed in is the exciter — an impulse plucks it, a gate edge strikes it, noise
//! bows it — and the bank rings at `freq` and decays. It is a filter, not a voice: silence
//! in, silence out.
//!
//! Lives here rather than in [`crate::ugen::filter`] because it has two consumers: the
//! `modal` UGen runs it over `ctx.state`, and doux's voice-insert stage runs it over its own
//! per-channel array. Same arrangement as [`crate::fastmath`] — the definition is here, the
//! host reaches in.
//!
//! # Partials
//!
//! Each mode `n` sits at `freq · rₙ`, and the three ratio sets are interpolated with
//! triangular weights over `structure·2 ∈ [0, 2]`:
//!
//! - string `1 2 3 4 5 6 7 8` — the harmonic series;
//! - bar `1 2.756 5.404 …` — free-free transverse modes (`βₙL = 4.730, 7.853, …` squared
//!   and normalized);
//! - bell `1 1.2 1.5 2 2.5 2.67 3 4` — minor-third bell partials.
//!
//! Everything between is a blend, so `structure` is one continuous knob from woody to
//! metallic to clangorous.
//!
//! # Decay
//!
//! Mode `n`'s ring time is `Tₙ = decay / rₙ^p` with `p = 2 − 1.75·bright`, so brightness is
//! exactly "how much faster the upper modes die". A resonator at `fc` ringing for `T` has
//! `Q = π·fc·T/ln(1000)`, hence the damping
//!
//! ```text
//! kₙ = ln(1000) · rₙ^(p−1) / (π · freq · decay)
//! ```
//!
//! clamped into `[1e-4, 2]` so the ring is always finite and the filter always damped.
//!
//! # Level
//!
//! Striking a body harder rings it louder; striking a *less damped* body rings it LONGER,
//! not louder. So each mode's gain is a constant [`STRIKE`], independent of its damping —
//! the decay knob buys ring time, not loudness. The price is that a raw TPT bandpass tap
//! peaks at `1/k`, so a sustained tone parked on a mode is amplified by `STRIKE/k`, which
//! for a 20 s ring is enormous. That build-up is what a real resonator does, but it is
//! capped at [`RESGAIN`] so it can never run away. A mode whose frequency approaches Nyquist
//! fades out over the last `0.05·sr` rather than piling up on the clamp, so a high `freq`
//! thins the bank out instead of turning it into a blob.
//!
//! Ported from the Faust modal resonator in the `rcs` synthesizer, with the per-sample
//! `sin`/`cos` replaced by cached coefficients (see [`tick`]).

use core::f32::consts::PI;

use crate::fastmath::{fast_tan, powf};
use crate::ugen::flush;

/// Modes in the bank. Eight is enough to read as a body without the ratio tables becoming
/// guesswork above the bar series' published roots.
pub const MODES: usize = 8;

/// Flat state a [`tick`] call owns: four coefficient-cache keys, then `s1, s2, a1, a2, a3,
/// gain` per mode. Callers hand in a slice at least this long, zero-filled when fresh.
pub const STATE_SLOTS: usize = KEYS + MODES * PER_MODE;

/// Cache keys: `freq` (biased), `decay`, `structure`, `bright`.
const KEYS: usize = 4;
/// Per mode: two integrators, three TPT coefficients, one output gain.
const PER_MODE: usize = 6;

/// Per-mode gain for a struck excitation, deliberately independent of damping.
const STRIKE: f32 = 0.15;
/// Ceiling on sustained resonant build-up (`STRIKE/k`, +35 dB).
const RESGAIN: f32 = 60.0;
/// `ln(1000)` — the 60 dB in the `Q ↔ ring time` identity.
const LN1000: f32 = 6.907_755;
/// Cap on the TPT frequency coefficient, as the SVF caps it: above Nyquist `tan` goes
/// negative and the filter latches a NaN.
const G_MAX: f32 = 16.0;

/// Frequency limits. The low end keeps `k`'s `1/freq` finite; the high end is where the
/// Nyquist fade has already emptied the bank.
const FREQ_MIN: f32 = 20.0;
const FREQ_MAX: f32 = 20_000.0;
/// Ring-time limits, in seconds. The low end keeps `k`'s `1/decay` finite.
const DECAY_MIN: f32 = 0.05;
const DECAY_MAX: f32 = 20.0;

const STRING: [f32; MODES] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
const BAR: [f32; MODES] = [
    1.0, 2.756, 5.404, 8.933, 13.344, 18.638, 24.814, 31.874,
];
const BELL: [f32; MODES] = [1.0, 1.2, 1.5, 2.0, 2.5, 2.67, 3.0, 4.0];
/// `1/√(n+1)` — rolls the bank off with mode number so the top modes colour rather than
/// dominate.
const AMP: [f32; MODES] = [
    1.0, 0.707, 0.577, 0.5, 0.447, 0.408, 0.378, 0.354,
];

/// Ring one sample of `x` through the bank and return the summed modes.
///
/// `freq` is the fundamental in Hz, `decay` mode 1's ring time in seconds, `structure` the
/// string → bar → bell morph and `bright` how long the upper modes ring relative to mode 1;
/// the last two run `0..1`.
///
/// Coefficients cost eight `fast_tan` and eight `powf`, so they are cached and recomputed
/// only on the sample where a parameter actually changes — change-detect, never
/// block-latched, so audio-rate modulation still tracks exactly (at that price). The
/// integrators are deliberately *not* cleared on a recompute: zeroing them would silence the
/// ring every time a parameter moved, and a TPT filter takes a coefficient change without a
/// click.
///
/// # Panics
///
/// Panics if `state` is shorter than [`STATE_SLOTS`].
// Not `.clamp()`: `.max().min()` suppresses a NaN parameter instead of propagating it, which
// would latch the whole bank silent (see `svf_taps`).
#[allow(clippy::manual_clamp)]
pub fn tick(
    state: &mut [f32],
    x: f32,
    freq: f32,
    decay: f32,
    structure: f32,
    bright: f32,
    sr: f32,
) -> f32 {
    let f0 = freq.max(FREQ_MIN).min(FREQ_MAX);
    let ring = decay.max(DECAY_MIN).min(DECAY_MAX);
    let s = structure.max(0.0).min(1.0) * 2.0; // 0 string, 1 bar, 2 bell
    let b = bright.max(0.0).min(1.0);

    // `f0` carries the bias: it is clamped ≥ 20, so the stored key is ≥ 21 and the zero-filled
    // fresh state can never alias a valid one. That first forced recompute is what
    // initializes the raw secondary keys below.
    if state[0] != f0 + 1.0 || state[1] != ring || state[2] != s || state[3] != b {
        state[0] = f0 + 1.0;
        state[1] = ring;
        state[2] = s;
        state[3] = b;
        recompute(state, f0, ring, s, b, sr);
    }

    let mut y = 0.0;
    for m in 0..MODES {
        let base = KEYS + m * PER_MODE;
        let (a1, a2, a3, gain) = (
            state[base + 2],
            state[base + 3],
            state[base + 4],
            state[base + 5],
        );
        let ic1 = state[base];
        let ic2 = state[base + 1];
        let v3 = x - ic2;
        let v1 = a1 * ic1 + a2 * v3;
        let v2 = ic2 + a2 * ic1 + a3 * v3;
        state[base] = flush(2.0 * v1 - ic1);
        state[base + 1] = flush(2.0 * v2 - ic2);
        y += gain * v1; // v1 is the bandpass tap
    }
    y
}

/// Retune every mode. Called only from the cache miss in [`tick`]; the parameters arrive
/// already clamped.
#[allow(clippy::manual_clamp)]
fn recompute(state: &mut [f32], f0: f32, ring: f32, s: f32, b: f32, sr: f32) {
    let p = 2.0 - 1.75 * b; // upper-mode decay exponent
    let top = 0.45 * sr;
    for m in 0..MODES {
        // Triangular weights over `s`: at s = 0 only STRING contributes, at 1 only BAR, at 2
        // only BELL, and in between exactly two do.
        let ratio = (1.0 - s.abs()).max(0.0) * STRING[m]
            + (1.0 - (s - 1.0).abs()).max(0.0) * BAR[m]
            + (1.0 - (s - 2.0).abs()).max(0.0) * BELL[m];

        let raw = f0 * ratio;
        let fc = raw.max(FREQ_MIN).min(top);
        // A mode crossing Nyquist fades out over the last 0.05·sr instead of piling onto the
        // clamp with its siblings.
        let fade = ((top - raw) / (0.05 * sr)).max(0.0).min(1.0);
        let k = (LN1000 * powf(ratio, p - 1.0) / (PI * f0 * ring))
            .max(1e-4)
            .min(2.0);
        let g = fast_tan(PI * fc / sr).max(0.0).min(G_MAX);
        let a1 = 1.0 / (1.0 + g * (g + k));

        let base = KEYS + m * PER_MODE;
        state[base + 2] = a1;
        state[base + 3] = g * a1;
        state[base + 4] = g * (g * a1);
        state[base + 5] = STRIKE.min(RESGAIN * k) * AMP[m] * fade;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f32 = 48_000.0;

    /// A bank struck once by a single full-scale sample. Nothing is fed in afterwards, so
    /// everything the caller measures is the ring.
    fn struck(freq: f32, decay: f32, structure: f32, bright: f32) -> [f32; STATE_SLOTS] {
        let mut state = [0.0; STATE_SLOTS];
        tick(&mut state, 1.0, freq, decay, structure, bright, SR);
        state
    }

    fn run(state: &mut [f32; STATE_SLOTS], freq: f32, decay: f32, n: usize) -> f32 {
        let mut y = 0.0;
        for _ in 0..n {
            y = tick(state, 0.0, freq, decay, 0.0, 0.5, SR);
        }
        y
    }

    // It is a filter, not a voice: with nothing fed in it must stay silent, however long the
    // decay and wherever the morph sits.
    #[test]
    fn silent_without_an_exciter() {
        let mut state = [0.0; STATE_SLOTS];
        for n in 0..(2 * SR as usize) {
            let t = n as f32 / SR;
            let sweep = (core::f32::consts::TAU * 3.0 * t).sin().abs();
            let y = tick(&mut state, 0.0, 20.0 + 4000.0 * sweep, 20.0, sweep, sweep, SR);
            assert_eq!(y, 0.0, "rang without an exciter at sample {n}");
        }
    }

    /// Rising zero-crossings per second of the ring, measured over the half second starting
    /// one second after the strike. `bright` 0 gives mode 2 a 0.5 s ring against mode 1's
    /// 2 s, so by then only mode 1 is left and this is its frequency.
    fn ring_frequency(freq: f32) -> u32 {
        let mut state = struck(freq, 2.0, 0.0, 0.0);
        let mut prev = 0.0;
        for _ in 0..(SR as usize) {
            prev = tick(&mut state, 0.0, freq, 2.0, 0.0, 0.0, SR);
        }
        let mut crossings = 0u32;
        for _ in 0..(SR as usize / 2) {
            let now = tick(&mut state, 0.0, freq, 2.0, 0.0, 0.0, SR);
            if prev < 0.0 && now >= 0.0 {
                crossings += 1;
            }
            prev = now;
        }
        crossings * 2
    }

    #[test]
    fn the_ring_sits_at_freq() {
        let base = ring_frequency(220.0);
        let up = ring_frequency(440.0);
        assert!((215..=225).contains(&base), "220 Hz rang at {base} Hz");
        assert!((430..=450).contains(&up), "440 Hz rang at {up} Hz");
    }

    // The decay knob is mode 1's ring time, so a long setting is still audible long after a
    // short one has died away.
    #[test]
    fn the_decay_knob_sets_the_ring_time() {
        fn peak_after(decay: f32, seconds: usize) -> f32 {
            let mut state = struck(220.0, decay, 0.0, 0.5);
            run(&mut state, 220.0, decay, seconds * SR as usize);
            let mut peak = 0.0f32;
            for _ in 0..(SR as usize / 10) {
                peak = peak.max(tick(&mut state, 0.0, 220.0, decay, 0.0, 0.5, SR).abs());
            }
            peak
        }
        let short = peak_after(0.2, 1);
        let long = peak_after(10.0, 1);
        assert!(
            long > short * 100.0,
            "a 10 s ring should outlast a 0.2 s one a second on: short={short} long={long}"
        );
    }

    // Every mode stays finite and bounded with a hot input while all four parameters sweep
    // their full range — including a `freq` high enough to push the upper modes past Nyquist,
    // where they must fade out rather than pile up.
    #[test]
    fn finite_under_a_hot_input_and_a_full_sweep() {
        let mut state = [0.0; STATE_SLOTS];
        for n in 0..(4 * SR as usize) {
            let t = n as f32 / SR;
            let audio = (core::f32::consts::TAU * 220.0 * t).sin();
            let freq = 20.0 + 19_980.0 * (core::f32::consts::TAU * 3.0 * t).sin().abs();
            let structure = (core::f32::consts::TAU * 7.0 * t).sin() * 0.5 + 0.5;
            let bright = (core::f32::consts::TAU * 11.0 * t).sin() * 0.5 + 0.5;
            let y = tick(&mut state, audio, freq, 20.0, structure, bright, SR);
            assert!(y.is_finite(), "non-finite at sample {n}");
            assert!(y.abs() < 128.0, "runaway resonance at sample {n}: {y}");
        }
    }

    // Driving a mode at its own frequency is the one case that builds up rather than decays.
    // It must genuinely resonate — far above unity — and still land under the RESGAIN ceiling.
    #[test]
    fn a_tone_parked_on_a_mode_resonates_but_stays_capped() {
        let mut state = [0.0; STATE_SLOTS];
        let total = 10 * SR as usize; // several ring times, so it reaches steady state
        let mut peak = 0.0f32;
        for n in 0..total {
            let x = (core::f32::consts::TAU * 220.0 * n as f32 / SR).sin();
            let y = tick(&mut state, x, 220.0, 2.0, 0.0, 0.5, SR);
            if n >= total / 2 {
                peak = peak.max(y.abs());
            }
        }
        assert!(peak > 5.0, "a tone on the mode barely resonated: {peak}");
        assert!(peak < RESGAIN, "resonant build-up passed the ceiling: {peak}");
    }

    // Out-of-range and non-finite parameters must be absorbed, not latched: NaN through
    // `.max().min()` collapses to a bound rather than poisoning every coefficient.
    #[test]
    fn hostile_parameters_are_absorbed() {
        for (freq, decay, structure, bright) in [
            (0.0, 0.0, -5.0, -5.0),
            (f32::NAN, f32::NAN, f32::NAN, f32::NAN),
            (1e9, 1e9, 9.0, 9.0),
            (-440.0, -2.0, 0.5, 0.5),
        ] {
            let mut state = [0.0; STATE_SLOTS];
            for _ in 0..1000 {
                let y = tick(&mut state, 1.0, freq, decay, structure, bright, SR);
                assert!(y.is_finite(), "{freq}/{decay}/{structure}/{bright} went non-finite");
            }
        }
    }

    // The morph is continuous: mode 2 walks from the harmonic 2:1 through the bar's 2.756:1
    // to the bell's 1.2:1 without a jump.
    #[test]
    fn structure_morphs_the_partials() {
        let ratio = |s: f32| {
            let mut state = [0.0; STATE_SLOTS];
            tick(&mut state, 0.0, 220.0, 2.0, s, 0.5, SR);
            // Recover mode 2's tuning from its cached `g = tan(π·fc/sr)`.
            let a1 = state[KEYS + PER_MODE + 2];
            let a2 = state[KEYS + PER_MODE + 3];
            let g = a2 / a1;
            g.atan() * SR / PI / 220.0
        };
        assert!((ratio(0.0) - 2.0).abs() < 0.05, "string: {}", ratio(0.0));
        assert!((ratio(0.5) - 2.756).abs() < 0.05, "bar: {}", ratio(0.5));
        assert!((ratio(1.0) - 1.2).abs() < 0.05, "bell: {}", ratio(1.0));
    }
}
