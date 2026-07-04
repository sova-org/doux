//! Filters. One-pole `lpf`/`hpf` (the high-pass is the residual the low-pass leaves
//! behind, `hpf = in − lpf(in)`, so they share the same coefficient and z1 memory), and
//! the resonant second-order family `lpf2`/`hpf2`/`bpf`/`notch` — one shared TPT/Zavalishin
//! state-variable core ([`svf_taps`]) tapped four ways.

use core::f32::consts::{PI, SQRT_2, TAU};

use super::{signal, Arity, Category, InputDescriptor, TickCtx, UGen, Unit};

pub(super) static UGENS: &[UGen] = &[
    // lpf  ( in cutoff -- sig )  state: [z1]
    UGen { name: "lpf", category: Category::Filter, description: "One-pole low-pass — attenuates above the cutoff.",
           examples: &["110 saw 800 lpf 0.3 * out", "noise 1200 lpf 0.3 * out"], arity: Arity::Fixed(2),
           inputs: &[signal("in"), InputDescriptor { name: "cutoff", unit: Unit::Hz, range: (20.0, 20_000.0), default: 1_000.0 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 12, tick: tick_lpf },
    // hpf  ( in cutoff -- sig )  state: [z1]   one-pole high-pass = in − lpf(in)
    UGen { name: "hpf", category: Category::Filter, description: "One-pole high-pass — the residual of the low-pass; attenuates below the cutoff.",
           examples: &["110 saw 1500 hpf 0.3 * out", "noise 4000 hpf 0.3 * out"], arity: Arity::Fixed(2),
           inputs: &[signal("in"), InputDescriptor { name: "cutoff", unit: Unit::Hz, range: (20.0, 20_000.0), default: 1_000.0 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 12, tick: tick_hpf },
    // lpf2  ( in cutoff res -- sig )  state: [ic1 ic2]   resonant SVF low-pass tap
    UGen { name: "lpf2", category: Category::Filter, description: "Resonant two-pole low-pass (state-variable).",
           examples: &["110 saw 600 0.8 lpf2 0.3 * out", "110 saw  2 sine 200 2000 range  0.8 lpf2 0.3 * out"], arity: Arity::Fixed(3), inputs: SVF_INPUTS, outputs: 1,
           state_slots: 2, buffer_len: 0, cost: 16, tick: tick_lpf2 },
    // hpf2  ( in cutoff res -- sig )  state: [ic1 ic2]   resonant SVF high-pass tap
    UGen { name: "hpf2", category: Category::Filter, description: "Resonant two-pole high-pass (state-variable).",
           examples: &["noise 1200 0.8 hpf2 0.3 * out", "110 saw 1500 0.7 hpf2 0.3 * out"], arity: Arity::Fixed(3), inputs: SVF_INPUTS, outputs: 1,
           state_slots: 2, buffer_len: 0, cost: 16, tick: tick_hpf2 },
    // bpf  ( in cutoff res -- sig )  state: [ic1 ic2]   resonant SVF band-pass tap
    UGen { name: "bpf", category: Category::Filter, description: "Resonant two-pole band-pass (state-variable).",
           examples: &["noise 1000 0.8 bpf 0.4 * out", "110 saw  1 sine 300 1800 range  0.8 bpf 0.4 * out"], arity: Arity::Fixed(3), inputs: SVF_INPUTS, outputs: 1,
           state_slots: 2, buffer_len: 0, cost: 16, tick: tick_bpf },
    // notch  ( in cutoff res -- sig )  state: [ic1 ic2]   resonant SVF notch (low + high)
    UGen { name: "notch", category: Category::Filter, description: "Resonant two-pole band-reject / notch (state-variable).",
           examples: &["noise 1000 0.7 notch 0.3 * out", "110 saw 800 0.8 notch 0.3 * out"], arity: Arity::Fixed(3), inputs: SVF_INPUTS, outputs: 1,
           state_slots: 2, buffer_len: 0, cost: 16, tick: tick_notch },
    // svf  ( in cutoff res -- lp bp hp notch )   the shared core tapped four ways at once: one
    // instance, one pair of integrators, four correlated outputs (vs four separate filters).
    UGen { name: "svf", category: Category::Filter, description: "State-variable filter core — low-, band-, high-pass and notch taps at once.",
           examples: &["[ 110 saw 800 0.7 svf ] 0 nth 0.3 * out", "[ noise 1500 0.8 svf ] 2 nth 0.3 * out"], arity: Arity::Fixed(3), inputs: SVF_INPUTS, outputs: 4,
           state_slots: 2, buffer_len: 0, cost: 16, tick: tick_svf },
    // lag  ( in time -- sig )  state: [z1]   one-pole slew limiter; `time` is the smoothing
    // time constant in seconds (0 ⇒ pass-through). Smooths control signals (portamento).
    UGen { name: "lag", category: Category::Filter, description: "One-pole slew — eases a signal toward its target over `time` seconds (portamento; 0 = passthrough).",
           examples: &["noise 0.002 lag 300 * 400 +  sine 0.2 * out", "2 impulse 0.1 lag 440 sine * 0.3 * out"], arity: Arity::Fixed(2),
           inputs: &[signal("in"), InputDescriptor { name: "time", unit: Unit::Seconds, range: (0.0, 10.0), default: 0.1 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 12, tick: tick_lag },
    // apf  ( in cutoff -- sig )  state: [w1]   first-order allpass: unity magnitude, frequency-
    // dependent phase. Diffuses transients and builds phasers when summed with the dry signal.
    UGen { name: "apf", category: Category::Filter, description: "First-order allpass — flat magnitude, frequency-dependent phase (phasers, diffusion).",
           examples: &["110 saw 0.3 *  dup 600 apf  +  0.5 * out", "2 impulse  ( 700 apf ) 8 times  0.4 * out"], arity: Arity::Fixed(2),
           inputs: &[signal("in"), InputDescriptor { name: "cutoff", unit: Unit::Hz, range: (20.0, 20_000.0), default: 1_000.0 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 12, tick: tick_apf },
    // moog  ( in cutoff res -- sig )  state: [y1 y2 y3 y4]   four cascaded one-poles with
    // tanh-bounded feedback from the last stage — the classic ladder slope, self-oscillating
    // toward res 1, unconditionally stable.
    UGen { name: "moog", category: Category::Filter, description: "Moog-style ladder low-pass — four cascaded poles with resonant feedback; self-oscillates as `res` nears 1.",
           examples: &["110 saw 800 0.6 moog 0.3 * out", "110 saw  0.3 sine 2000 * 2500 +  0.7 moog 0.2 * out"], arity: Arity::Fixed(3), inputs: SVF_INPUTS, outputs: 1,
           state_slots: 4, buffer_len: 0, cost: 24, tick: tick_moog },
    // ringz  ( in freq decay -- sig )  state: [y1 y2]   two-pole ringing resonator: every
    // input sample strikes a damped sinusoid at `freq` ringing 60 dB down over `decay` seconds.
    UGen { name: "ringz", category: Category::Filter, description: "Ringing resonator — strikes a damped sinusoid at `freq`, ringing out over `decay` seconds (mallets, modal bodies).",
           examples: &["4 impulse 880 0.3 ringz 0.5 * out", "noise 1200 0.2 ringz 0.1 * out"], arity: Arity::Fixed(3),
           inputs: &[signal("in"),
                     InputDescriptor { name: "freq", unit: Unit::Hz, range: (20.0, 20_000.0), default: 440.0 },
                     InputDescriptor { name: "decay", unit: Unit::Seconds, range: (0.0, 10.0), default: 0.3 }],
           outputs: 1, state_slots: 2, buffer_len: 0, cost: 20, tick: tick_ringz },
    // slew  ( in up down -- sig )  state: [y1]   rate limiter: the output chases the input,
    // rising at most `up` and falling at most `down` units per second (lag's linear cousin).
    UGen { name: "slew", category: Category::Filter, description: "Slew limiter — the output chases the input, bounded to `up`/`down` units per second.",
           examples: &["8 impulse 4 4 slew 440 sine * 0.3 * out", "noise 8 8 slew 0.3 * out"], arity: Arity::Fixed(3),
           inputs: &[signal("in"),
                     InputDescriptor { name: "up", unit: Unit::Ratio, range: (0.0, 10_000.0), default: 100.0 },
                     InputDescriptor { name: "down", unit: Unit::Ratio, range: (0.0, 10_000.0), default: 100.0 }],
           outputs: 1, state_slots: 1, buffer_len: 0, cost: 4, tick: tick_slew },
    // dcblock  ( in -- sig )  state: [x1 y1]   one-zero/one-pole DC blocker with a fixed
    // ~10 Hz pole — removes the offset a unipolar modulator or asymmetric waveshaper leaves.
    UGen { name: "dcblock", category: Category::Filter, description: "DC blocker — removes the constant offset, leaving the audio band untouched.",
           examples: &["110 saw 0.5 * 0.3 + dcblock 0.3 * out", "noise abs dcblock 0.5 * out"], arity: Arity::Fixed(1), inputs: &[signal("in")], outputs: 1,
           state_slots: 2, buffer_len: 0, cost: 4, tick: tick_dcblock },
    // peak ( in freq gain q -- sig )  state: [z1 z2]   RBJ peaking EQ — boost/cut a band
    UGen { name: "peak", category: Category::Filter, description: "Peaking EQ — boosts or cuts a band by `gain` dB at `freq`, width set by `q` (RBJ biquad).",
           examples: &["noise 0.3 *  1200 6 2 peak  0.5 * out", "440 sine 0.3 *  600 -12 3 peak  out"], arity: Arity::Fixed(4),
           inputs: &[signal("in"), FREQ_INPUT, GAIN_INPUT, Q_INPUT], outputs: 1,
           state_slots: 2, buffer_len: 0, cost: 28, tick: tick_peak },
    // lowshelf ( in freq gain -- sig )  state: [z1 z2]   RBJ low shelf — lift/dip below `freq`
    UGen { name: "lowshelf", category: Category::Filter, description: "Low shelf — lifts or dips everything below `freq` by `gain` dB (RBJ biquad, fixed slope).",
           examples: &["110 saw 0.3 *  200 9 lowshelf  out", "noise 0.2 *  400 -12 lowshelf  0.5 * out"], arity: Arity::Fixed(3),
           inputs: &[signal("in"), FREQ_INPUT, GAIN_INPUT], outputs: 1,
           state_slots: 2, buffer_len: 0, cost: 30, tick: tick_lowshelf },
    // highshelf ( in freq gain -- sig )  state: [z1 z2]   RBJ high shelf — lift/dip above `freq`
    UGen { name: "highshelf", category: Category::Filter, description: "High shelf — lifts or dips everything above `freq` by `gain` dB (RBJ biquad, fixed slope).",
           examples: &["110 saw 0.3 *  3000 9 highshelf  out", "noise 0.2 *  5000 -18 highshelf  out"], arity: Arity::Fixed(3),
           inputs: &[signal("in"), FREQ_INPUT, GAIN_INPUT], outputs: 1,
           state_slots: 2, buffer_len: 0, cost: 30, tick: tick_highshelf },
    // reson ( in freq q -- sig )  state: [z1 z2]   RBJ band-pass (constant 0 dB peak), Q-set
    UGen { name: "reson", category: Category::Filter, description: "Resonant band-pass — a constant 0 dB peak at `freq`, sharpness set by `q` (RBJ biquad).",
           examples: &["noise  1500 12 reson  0.3 * out", "110 saw  800 20 reson  0.4 * out"], arity: Arity::Fixed(3),
           inputs: &[signal("in"), FREQ_INPUT, Q_INPUT], outputs: 1,
           state_slots: 2, buffer_len: 0, cost: 26, tick: tick_reson },
];

/// Shared signature for the resonant SVF family: a signal in, a cutoff in Hz, and
/// resonance as a 0..1 ratio. Declared once so all four taps read the same source of truth.
const SVF_INPUTS: &[InputDescriptor] = &[
    signal("in"),
    InputDescriptor { name: "cutoff", unit: Unit::Hz, range: (20.0, 20_000.0), default: 1_000.0 },
    InputDescriptor { name: "res", unit: Unit::Ratio, range: (0.0, 1.0), default: 0.3 },
];

/// Upper clamp on the SVF coefficient `g = tan(π·fc/sr)`. `tan` blows toward ±∞ as the
/// cutoff nears Nyquist; capping `g` keeps `a1 = 1/(1 + g·(g+k))` finite and NaN-free at
/// the boundary cases the harness pins. 16 ≈ cutoff 0.4965·sr.
const G_MAX: f32 = 16.0;
/// Resonance maps to the SVF damping `k = 1/Q`: res 0 → √2 (a flat Butterworth 2-pole),
/// res 1 → `K_MIN` (sharp, Q ≈ 10). `K_SPAN` is precomputed as a named f32 constant.
const K_MIN: f32 = 0.1;
const K_SPAN: f32 = SQRT_2 - K_MIN;

/// Shared `freq` input for the RBJ biquads (`peak`/`lowshelf`/`highshelf`/`reson`).
const FREQ_INPUT: InputDescriptor =
    InputDescriptor { name: "freq", unit: Unit::Hz, range: (20.0, 20_000.0), default: 1_000.0 };
/// Boost/cut in decibels for `peak`/`lowshelf`/`highshelf` (0 dB = unity).
const GAIN_INPUT: InputDescriptor =
    InputDescriptor { name: "gain", unit: Unit::None, range: (-24.0, 24.0), default: 6.0 };
/// Resonance Q for `peak`/`reson` (higher = narrower); floored at `Q_MIN` in the tick.
const Q_INPUT: InputDescriptor =
    InputDescriptor { name: "q", unit: Unit::None, range: (0.1, 100.0), default: 1.0 };

/// RBJ biquads: lowest `q` — keeps `α = sin ω/(2q)` finite (q → 0 would give NaN coefficients).
const Q_MIN: f32 = 0.1;
/// RBJ biquads: `freq` is clamped to this fraction of the sample rate (musical sanity near Nyquist).
const NYQ_FRAC: f32 = 0.49;
/// RBJ shelf `α` factor at the fixed slope S = 1: `α = sin ω · (√2 / 2)`, as a named f32
/// constant.
const SHELF_ALPHA_K: f32 = SQRT_2 / 2.0;

fn tick_lpf(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    let fc = ctx.inputs[1].max(0.0);
    // One-pole low-pass coefficient a = 1 - e^{-2π·fc/sr}, clamped to a stable range.
    let a = (1.0 - (-TAU * fc / ctx.sr).exp()).clamp(0.0, 1.0);
    let y = ctx.state[0] + a * (x - ctx.state[0]);
    ctx.state[0] = y;
    out[0] = y;
}

fn tick_hpf(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    let fc = ctx.inputs[1].max(0.0);
    // Same one-pole low-pass coefficient and memory as `lpf`; the high-pass is the
    // residual the low-pass leaves behind: y = x - lowpass.
    let a = (1.0 - (-TAU * fc / ctx.sr).exp()).clamp(0.0, 1.0);
    ctx.state[0] = ctx.state[0] + a * (x - ctx.state[0]);
    out[0] = x - ctx.state[0];
}

/// The shared TPT/Zavalishin state-variable core. Advances the two integrator states
/// (`state[0]`/`state[1]`) one sample from `in`/`cutoff`/`res` and returns the
/// `(low, band, high)` taps; each `lpf2`/`hpf2`/`bpf`/`notch` tick selects from these, so the
/// filter math lives here once. Unconditionally stable for any `g > 0, k > 0`; the clamps
/// only keep `tan`'s Nyquist blow-up and NaN inputs finite.
// Not `.clamp()`: `clamp` preserves NaN, whereas `.max().min()` suppresses it. A NaN cutoff
// flows through `tan` here, so suppressing it with max/min is what keeps NaN out of the
// integrator state (see the boundary tests).
#[allow(clippy::manual_clamp)]
fn svf_taps(ctx: &mut TickCtx) -> (f32, f32, f32) {
    let x = ctx.inputs[0];
    let g = (PI * ctx.inputs[1] / ctx.sr).tan().max(0.0).min(G_MAX);
    let r = ctx.inputs[2].max(0.0).min(1.0);
    let k = SQRT_2 - K_SPAN * r;
    let ic1 = ctx.state[0];
    let ic2 = ctx.state[1];
    let a1 = 1.0 / (1.0 + g * (g + k));
    let a2 = g * a1;
    let a3 = g * a2;
    let v3 = x - ic2;
    let v1 = a1 * ic1 + a2 * v3;
    let v2 = ic2 + a2 * ic1 + a3 * v3;
    ctx.state[0] = 2.0 * v1 - ic1;
    ctx.state[1] = 2.0 * v2 - ic2;
    let low = v2;
    let band = v1;
    let high = x - k * v1 - v2;
    (low, band, high)
}

fn tick_lpf2(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = svf_taps(ctx).0;
}

fn tick_hpf2(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = svf_taps(ctx).2;
}

fn tick_bpf(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = svf_taps(ctx).1;
}

fn tick_notch(ctx: &mut TickCtx, out: &mut [f32]) {
    let (low, _band, high) = svf_taps(ctx);
    out[0] = low + high;
}

fn tick_svf(ctx: &mut TickCtx, out: &mut [f32]) {
    // One core, four correlated taps — `notch` is `low + high`, exactly as `tick_notch` derives it.
    let (low, band, high) = svf_taps(ctx);
    out[0] = low; // lp
    out[1] = band; // bp
    out[2] = high; // hp
    out[3] = low + high; // notch
}

fn tick_lag(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    let t = ctx.inputs[1].max(0.0);
    // One-pole smoother coefficient from a time constant in seconds: a = 1 - e^{-1/(t·sr)}
    // (t = 0 ⇒ a = 1, instant). Same NaN-free clamp story as `lpf` (the `.max(0)` de-NaNs t).
    let a = (1.0 - (-1.0 / (t * ctx.sr)).exp()).clamp(0.0, 1.0);
    let y = ctx.state[0] + a * (x - ctx.state[0]);
    ctx.state[0] = y;
    out[0] = y;
}

// Not `.clamp()`: `.max().min()` suppresses NaN so a NaN coefficient collapses to a bound
// instead of latching into the filter state, exactly as `svf_taps` does.
#[allow(clippy::manual_clamp)]
fn tick_apf(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    let fc = ctx.inputs[1].max(0.0);
    // First-order allpass, bilinear-mapped cutoff. DF-II: w = x − c·w₁; y = c·w + w₁.
    // Cap `tan` the same way the SVF caps `g` (`.max(0).min(G_MAX)`): above Nyquist `tan`
    // goes negative or blows toward ±∞, which drives the coefficient past the unit circle and
    // sends the allpass to a sticky NaN. Clamped, `t ∈ [0, G_MAX]` keeps `c` in (−1, 1].
    let t = (PI * fc / ctx.sr).tan().max(0.0).min(G_MAX);
    let c = (t - 1.0) / (t + 1.0);
    let w = x - c * ctx.state[0];
    out[0] = c * w + ctx.state[0];
    ctx.state[0] = w;
}

// Not `.clamp()` on res: `.max().min()` suppresses NaN (see `svf_taps`).
#[allow(clippy::manual_clamp)]
fn tick_moog(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    let fc = ctx.inputs[1].max(0.0);
    // The exact `lpf` one-pole coefficient, shared by all four stages.
    let a = (1.0 - (-TAU * fc / ctx.sr).exp()).clamp(0.0, 1.0);
    // res 0..1 maps to feedback 0..4; at k = 4 the linear ladder is marginally stable and the
    // tanh on the input bounds it into self-oscillation instead of runaway.
    let k = 4.0 * ctx.inputs[2].max(0.0).min(1.0);
    let drive = (x - k * ctx.state[3]).tanh();
    let y1 = ctx.state[0] + a * (drive - ctx.state[0]);
    let y2 = ctx.state[1] + a * (y1 - ctx.state[1]);
    let y3 = ctx.state[2] + a * (y2 - ctx.state[2]);
    let y4 = ctx.state[3] + a * (y3 - ctx.state[3]);
    ctx.state[0] = y1;
    ctx.state[1] = y2;
    ctx.state[2] = y3;
    ctx.state[3] = y4;
    out[0] = y4;
}

fn tick_ringz(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    let theta = TAU * ctx.inputs[1] / ctx.sr;
    // Pole radius for a 60 dB ring over `decay` seconds: r = 0.001^{1/(decay·sr)}.
    // decay 0 ⇒ exponent +∞ ⇒ r = 0 (a dead filter, NaN-free); the `.max(0)` de-NaNs decay.
    let decay = ctx.inputs[2].max(0.0);
    let r = 0.001f32.powf(1.0 / (decay * ctx.sr));
    let b1 = 2.0 * r * theta.cos();
    let b2 = -(r * r);
    let y1 = ctx.state[0];
    let y2 = ctx.state[1];
    let y0 = x + b1 * y1 + b2 * y2;
    // The (1 − z⁻²)/2 numerator centers the passband gain independent of `freq`.
    out[0] = 0.5 * (y0 - y2);
    ctx.state[1] = y1;
    ctx.state[0] = y0;
}

// Not `.clamp()`: `.max().min()` suppresses a NaN step bound (see `svf_taps`).
#[allow(clippy::manual_clamp)]
fn tick_slew(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    let up = ctx.inputs[1].max(0.0) / ctx.sr;
    let down = ctx.inputs[2].max(0.0) / ctx.sr;
    let step = (x - ctx.state[0]).max(-down).min(up);
    let y = ctx.state[0] + step;
    ctx.state[0] = y;
    out[0] = y;
}

/// `dcblock`'s pole frequency in Hz. Fixed (not an input): the only musical choice is "below
/// the audio band", and baking it keeps the word arity-1 and the coefficient transcendental-free.
const DC_FC: f32 = 10.0;

fn tick_dcblock(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    // One-zero/one-pole: y = x − x₁ + r·y₁, with r = 1 − 2π·fc/sr (the first-order
    // approximation of the pole at fc — exact enough this far below the band).
    let r = 1.0 - TAU * DC_FC / ctx.sr;
    let y = x - ctx.state[0] + r * ctx.state[1];
    ctx.state[0] = x;
    ctx.state[1] = y;
    out[0] = y;
}

/// Transposed Direct Form II biquad step, shared by the RBJ filters
/// (`peak`/`lowshelf`/`highshelf`/`reson`) — the biquad twin of `svf_taps`. Takes the five
/// a0-normalized coefficients, advances the two state slots one sample, and returns the output.
/// Each filter computes its RBJ-cookbook coefficients then calls this, so the recurrence lives
/// here once.
fn biquad_step(ctx: &mut TickCtx, b0: f32, b1: f32, b2: f32, a1: f32, a2: f32) -> f32 {
    let x = ctx.inputs[0];
    let z1 = ctx.state[0];
    let z2 = ctx.state[1];
    let y = b0 * x + z1;
    ctx.state[0] = b1 * x - a1 * y + z2;
    ctx.state[1] = b2 * x - a2 * y;
    y
}

// Not `.clamp()`: `.max().min()` suppresses NaN (as `svf_taps`).
#[allow(clippy::manual_clamp)]
fn tick_peak(ctx: &mut TickCtx, out: &mut [f32]) {
    let sr = ctx.sr;
    let f = ctx.inputs[1].max(0.0).min(NYQ_FRAC * sr);
    let a = 10.0f32.powf(ctx.inputs[2] / 40.0); // A = 10^(gain/40)
    let q = ctx.inputs[3].max(Q_MIN);
    let w0 = TAU * f / sr;
    let cw = w0.cos();
    let alpha = w0.sin() / (2.0 * q);
    let alpha_a = alpha * a; // α·A
    let alpha_div_a = alpha / a; // α/A
    let a0 = 1.0 + alpha_div_a;
    let neg2cw = (-2.0 * cw) / a0; // shared by b1 and a1
    let b0 = (1.0 + alpha_a) / a0;
    let b2 = (1.0 - alpha_a) / a0;
    let a2 = (1.0 - alpha_div_a) / a0;
    out[0] = biquad_step(ctx, b0, neg2cw, b2, neg2cw, a2);
}

#[allow(clippy::manual_clamp)]
fn tick_lowshelf(ctx: &mut TickCtx, out: &mut [f32]) {
    let sr = ctx.sr;
    let f = ctx.inputs[1].max(0.0).min(NYQ_FRAC * sr);
    let a = 10.0f32.powf(ctx.inputs[2] / 40.0);
    let w0 = TAU * f / sr;
    let cw = w0.cos();
    let alpha = w0.sin() * SHELF_ALPHA_K; // S = 1 fixed slope
    let am1 = a - 1.0;
    let ap1 = a + 1.0;
    let beta = 2.0 * a.sqrt() * alpha; // 2·√A·α
    let am1_cw = am1 * cw;
    let ap1_cw = ap1 * cw;
    let a0 = ap1 + am1_cw + beta;
    let b0 = (a * (ap1 - am1_cw + beta)) / a0;
    let b1 = (2.0 * a * (am1 - ap1_cw)) / a0;
    let b2 = (a * (ap1 - am1_cw - beta)) / a0;
    let a1 = (-2.0 * (am1 + ap1_cw)) / a0;
    let a2 = (ap1 + am1_cw - beta) / a0;
    out[0] = biquad_step(ctx, b0, b1, b2, a1, a2);
}

#[allow(clippy::manual_clamp)]
fn tick_highshelf(ctx: &mut TickCtx, out: &mut [f32]) {
    let sr = ctx.sr;
    let f = ctx.inputs[1].max(0.0).min(NYQ_FRAC * sr);
    let a = 10.0f32.powf(ctx.inputs[2] / 40.0);
    let w0 = TAU * f / sr;
    let cw = w0.cos();
    let alpha = w0.sin() * SHELF_ALPHA_K;
    let am1 = a - 1.0;
    let ap1 = a + 1.0;
    let beta = 2.0 * a.sqrt() * alpha;
    let am1_cw = am1 * cw;
    let ap1_cw = ap1 * cw;
    let a0 = ap1 - am1_cw + beta;
    let b0 = (a * (ap1 + am1_cw + beta)) / a0;
    let b1 = (-2.0 * a * (am1 + ap1_cw)) / a0;
    let b2 = (a * (ap1 + am1_cw - beta)) / a0;
    let a1 = (2.0 * (am1 - ap1_cw)) / a0;
    let a2 = (ap1 - am1_cw - beta) / a0;
    out[0] = biquad_step(ctx, b0, b1, b2, a1, a2);
}

#[allow(clippy::manual_clamp)]
fn tick_reson(ctx: &mut TickCtx, out: &mut [f32]) {
    let sr = ctx.sr;
    let f = ctx.inputs[1].max(0.0).min(NYQ_FRAC * sr);
    let q = ctx.inputs[2].max(Q_MIN);
    let w0 = TAU * f / sr;
    let cw = w0.cos();
    let alpha = w0.sin() / (2.0 * q);
    let a0 = 1.0 + alpha;
    // RBJ band-pass (constant 0 dB peak gain), normalized by a0; b1 is 0.
    let b0 = alpha / a0;
    let b2 = -alpha / a0;
    let a1 = (-2.0 * cw) / a0;
    let a2 = (1.0 - alpha) / a0;
    out[0] = biquad_step(ctx, b0, 0.0, b2, a1, a2);
}
