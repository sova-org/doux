//! Oscillators: `sine` (a pure partial), plus the band-limited `saw`, `pulse` (variable
//! `width`), `tri`, `varsaw` (saw↔triangle morph), and `blip` (impulse train). Each holds its
//! phase in `[0, 1)` in one state slot; the edge/corner shapes add a polyBLEP/polyBLAMP residual
//! ([`poly_blep`] for value steps, [`poly_blamp`] for slope corners) to the naive waveform to
//! suppress aliasing; `blip` is band-limited by construction (a closed-form harmonic sum).

use core::f32::consts::{PI, TAU};

use super::{wrap01, Arity, Category, InputDescriptor, TickCtx, UGen, Unit};
use crate::fastmath::sinf;

pub(super) static UGENS: &[UGen] = &[
    // sine ( freq -- sig )   state: [phase 0..1]
    UGen { name: "sine", category: Category::Oscillator, description: "Sine oscillator — a pure tone; frequency in Hz.",
           examples: &["440 sine 0.2 * out", "440  5 sine 6 * +  sine 0.2 * out", "[ 220 330 ] sine 0.2 * out"], arity: Arity::Fixed(1),
           inputs: &[InputDescriptor { name: "freq", unit: Unit::Hz, range: (20.0, 20_000.0), default: 440.0 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 12, tick: tick_sine },
    // saw  ( freq -- sig )   state: [phase 0..1]
    UGen { name: "saw", category: Category::Oscillator, description: "Sawtooth oscillator — band-limited (polyBLEP); frequency in Hz.",
           examples: &["110 saw 0.2 * out", "110 saw 600 0.8 lpf2 0.3 * out"], arity: Arity::Fixed(1),
           inputs: &[InputDescriptor { name: "freq", unit: Unit::Hz, range: (20.0, 20_000.0), default: 110.0 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 10, tick: tick_saw },
    // pulse ( freq width -- sig )   state: [phase 0..1]   band-limited variable-width pulse
    UGen { name: "pulse", category: Category::Oscillator, description: "Pulse oscillator — band-limited (polyBLEP), variable `width` duty; frequency in Hz.",
           examples: &["110 0.5 pulse 0.2 * out", "110  0.2 sine 0.4 * 0.5 +  pulse 0.2 * out"], arity: Arity::Fixed(2),
           inputs: &[InputDescriptor { name: "freq", unit: Unit::Hz, range: (20.0, 20_000.0), default: 110.0 },
                     InputDescriptor { name: "width", unit: Unit::Ratio, range: (0.0, 1.0), default: 0.5 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 16, tick: tick_pulse },
    // tri ( freq -- sig )   state: [phase 0..1]   band-limited triangle (polyBLAMP)
    UGen { name: "tri", category: Category::Oscillator, description: "Triangle oscillator — band-limited (polyBLAMP); frequency in Hz.",
           examples: &["220 tri 0.3 * out", "220 tri 1200 lpf 0.3 * out"], arity: Arity::Fixed(1),
           inputs: &[InputDescriptor { name: "freq", unit: Unit::Hz, range: (20.0, 20_000.0), default: 110.0 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 16, tick: tick_tri },
    // varsaw ( freq width -- sig )   state: [phase 0..1]   band-limited saw↔tri morph (polyBLAMP)
    UGen { name: "varsaw", category: Category::Oscillator, description: "Variable-slope saw↔triangle — band-limited (polyBLAMP); `width` sets the peak position.",
           examples: &["110 0.3 varsaw 0.2 * out", "110  0.1 sine 0.45 * 0.5 +  varsaw 0.2 * out"], arity: Arity::Fixed(2),
           inputs: &[InputDescriptor { name: "freq", unit: Unit::Hz, range: (20.0, 20_000.0), default: 110.0 },
                     InputDescriptor { name: "width", unit: Unit::Ratio, range: (0.0, 1.0), default: 0.5 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 18, tick: tick_varsaw },
    // blip ( freq nharm -- sig )   state: [phase 0..1]   band-limited impulse train (Dirichlet)
    UGen { name: "blip", category: Category::Oscillator, description: "Band-limited impulse train — `nharm` equal cosine harmonics of `freq`, clamped under Nyquist (SC Blip).",
           examples: &["220 8 blip 0.3 * out", "220  2 sine 20 * 22 +  blip 0.3 * out"], arity: Arity::Fixed(2),
           inputs: &[InputDescriptor { name: "freq", unit: Unit::Hz, range: (20.0, 20_000.0), default: 440.0 },
                     InputDescriptor { name: "nharm", unit: Unit::None, range: (1.0, 512.0), default: 8.0 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 24, tick: tick_blip },
];

/// Canonical 2-sample polyBLEP residual: the correction added to a naive waveform to
/// band-limit a unit value step (the saw wrap, the pulse edges). `t` is the phase in [0, 1),
/// `dt = |freq|/sr` the one-sample correction window.
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let x = t / dt;
        2.0 * x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + 2.0 * x + 1.0
    } else {
        0.0
    }
}

/// One third, as f32: the polyBLAMP cubic's scale.
const THIRD: f32 = 1.0 / 3.0;

/// Canonical polyBLAMP residual — the integral of [`poly_blep`] — used to band-limit a unit
/// slope corner (triangle/varsaw peaks and troughs). `t`, `dt` as in [`poly_blep`].
fn poly_blamp(t: f32, dt: f32) -> f32 {
    if t < dt {
        let x = t / dt - 1.0;
        let x3 = x * x * x;
        -THIRD * x3
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt + 1.0;
        let x3 = x * x * x;
        THIRD * x3
    } else {
        0.0
    }
}

fn tick_sine(ctx: &mut TickCtx, out: &mut [f32]) {
    let p = ctx.state[0];
    out[0] = sinf(p * TAU);
    ctx.state[0] = wrap01(p + ctx.inputs[0] / ctx.sr);
}

fn tick_saw(ctx: &mut TickCtx, out: &mut [f32]) {
    let p = ctx.state[0];
    let inc = ctx.inputs[0] / ctx.sr;
    let dt = inc.abs();
    out[0] = (2.0 * p - 1.0) - poly_blep(p, dt); // naive ramp minus the wrap-step residual
    ctx.state[0] = wrap01(p + inc);
}

// Not `.clamp()`: `.max().min()` suppresses a NaN width to a bound (`clamp` propagates it).
#[allow(clippy::manual_clamp)]
fn tick_pulse(ctx: &mut TickCtx, out: &mut [f32]) {
    let p = ctx.state[0];
    // Clamp width away from the degenerate extremes exactly like `varsaw`: at w ≤ 0 or ≥ 1 the
    // two BLEPs cancel and the output collapses to a silent DC plateau.
    let w = ctx.inputs[1].max(0.005).min(0.995);
    let inc = ctx.inputs[0] / ctx.sr;
    let dt = inc.abs();
    let naive = if p < w { 1.0 } else { -1.0 };
    let edge = wrap01(p - w); // the falling edge, mapped onto the wrap
    out[0] = naive + poly_blep(p, dt) - poly_blep(edge, dt);
    ctx.state[0] = wrap01(p + inc);
}

fn tick_tri(ctx: &mut TickCtx, out: &mut [f32]) {
    let p = ctx.state[0];
    let inc = ctx.inputs[0] / ctx.sr;
    let dt = inc.abs();
    let naive = if p < 0.5 { 4.0 * p - 1.0 } else { 3.0 - 4.0 * p }; // trough −1@0, peak +1@0.5
    let corr = 4.0 * dt * (poly_blamp(p, dt) - poly_blamp(wrap01(p + 0.5), dt));
    out[0] = naive + corr;
    ctx.state[0] = wrap01(p + inc);
}

// Not `.clamp()`: `.max().min()` suppresses a NaN width to a bound (`clamp` propagates it).
#[allow(clippy::manual_clamp)]
fn tick_varsaw(ctx: &mut TickCtx, out: &mut [f32]) {
    let p = ctx.state[0];
    let w = ctx.inputs[1].max(0.005).min(0.995); // clamp away from the degenerate-step extremes
    let inc = ctx.inputs[0] / ctx.sr;
    let dt = inc.abs();
    // naive var-triangle: trough −1@0, peak +1@w
    let naive = if p < w { 2.0 * p / w - 1.0 } else { 1.0 - 2.0 * (p - w) / (1.0 - w) };
    let s = 1.0 / w + 1.0 / (1.0 - w); // summed corner slope magnitude
    let corr = dt * s * (poly_blamp(p, dt) - poly_blamp(wrap01(p - w), dt));
    out[0] = naive + corr;
    ctx.state[0] = wrap01(p + inc);
}

/// The singularity guard for `blip`'s Dirichlet quotient: below this |sin(πφ)| the closed
/// form is replaced by its limit.
const BLIP_EPS: f32 = 1e-4;

// Not `.clamp()`: `.max().min()` suppresses a NaN harmonic count to a bound (`clamp` propagates it).
#[allow(clippy::manual_clamp)]
fn tick_blip(ctx: &mut TickCtx, out: &mut [f32]) {
    // Band-limited impulse train via the closed-form harmonic sum (Dirichlet kernel):
    //   Σ_{k=1..N} cos(2πkφ) = (sin((2N+1)πφ) / sin(πφ) − 1) / 2,
    // normalized by N so the peak is exactly 1 (at φ = 0, where all harmonics align). N is
    // the requested count floored and clamped under Nyquist, so the sum can never alias.
    // Near the kernel's removable singularity (sin(πφ) ≈ 0) the quotient is replaced by its
    // limit 1.
    let p = ctx.state[0];
    let freq = ctx.inputs[0];
    let maxh = (ctx.sr / (2.0 * freq.abs())).floor(); // harmonics that fit under Nyquist
    let n = ctx.inputs[1].floor().min(maxh).max(1.0); // at least the fundamental
    let theta = p * PI;
    let denom = sinf(theta);
    let num = sinf((2.0 * n + 1.0) * theta);
    out[0] = if denom.abs() < BLIP_EPS {
        1.0
    } else {
        (num / denom - 1.0) / (2.0 * n)
    };
    ctx.state[0] = wrap01(p + freq / ctx.sr);
}
