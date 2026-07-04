//! Waveshapers / saturation / degradation: `tanh`, `distort`, `crush`, `decim`.

use super::{signal, Arity, Category, InputDescriptor, Rate, TickCtx, UGen, Unit};

/// `distort`'s drive scale: `amount` (0..1) maps to a `tanh` input gain of `1 + amount·DRIVE`,
/// so amount 0 is gentle and amount 1 hits ~10× into saturation. A taste constant.
const DRIVE: f32 = 9.0;

pub(super) static UGENS: &[UGen] = &[
    // tanh ( in -- out )   soft saturation; drive with `in k * tanh`
    UGen { name: "tanh", category: Category::Distortion, description: "Hyperbolic-tangent soft saturation.",
           examples: &["440 sine 4 * tanh 0.3 * out", "110 saw 6 * tanh 0.25 * out"], arity: Arity::Fixed(1), inputs: &[signal("in")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_tanh },
    // distort ( in amount -- out )   tanh saturation with an `amount`-driven gain (0 ⇒ soft)
    UGen { name: "distort", category: Category::Distortion, description: "tanh saturation driven by `amount` (0 = gentle).",
           examples: &["110 saw 0.6 distort 0.3 * out", "220 saw 0.9 distort 0.25 * out", "440 sine  2 sine 0.5 * 0.5 +  distort 0.3 * out"], arity: Arity::Fixed(2),
           inputs: &[signal("in"), InputDescriptor { name: "amount", unit: Unit::Ratio, range: (0.0, 1.0), default: 0.0, rate: Rate::Audio }],
           outputs: 1, state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 12, tick: tick_distort },
    // crush ( in bits -- out )   quantize the amplitude to `bits` of resolution
    UGen { name: "crush", category: Category::Distortion, description: "Bit crusher — quantizes the signal to `bits` of amplitude resolution.",
           examples: &["440 sine 4 crush 0.2 * out", "110 saw 3 crush 0.25 * out", "440 sine  1 phasor 6 * 2 +  crush 0.2 * out"], arity: Arity::Fixed(2),
           inputs: &[signal("in"), InputDescriptor { name: "bits", unit: Unit::Ratio, range: (1.0, 24.0), default: 8.0, rate: Rate::Audio }],
           outputs: 1, state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 12, tick: tick_crush },
    // decim ( in rate -- out )  state: [phase held]   sample-and-hold downsampler
    UGen { name: "decim", category: Category::Distortion, description: "Sample-rate decimator — holds the input, re-sampling it `rate` times per second (aliasing as color).",
           examples: &["440 sine 3000 decim 0.2 * out", "noise 8000 decim 0.2 * out", "220 saw  1 phasor 6000 * 1000 +  decim 0.2 * out"], arity: Arity::Fixed(2),
           inputs: &[signal("in"), InputDescriptor { name: "rate", unit: Unit::Hz, range: (0.0, 48_000.0), default: 8_000.0, rate: Rate::Audio }],
           outputs: 1, state_slots: 2, buffer_len: 0, rate: Rate::Audio, cost: 4, tick: tick_decim },
];

fn tick_tanh(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].tanh();
}

fn tick_distort(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = (ctx.inputs[0] * (1.0 + ctx.inputs[1] * DRIVE)).tanh();
}

// Not `.clamp()`: `.max().min()` mirrors the JIT's NaN-suppressing `arf_maxf`/`arf_minf` shims
// (a NaN bit-depth must collapse identically on both backends), as the filters do.
#[allow(clippy::manual_clamp)]
fn tick_crush(ctx: &mut TickCtx, out: &mut [f32]) {
    // Quantize to 2^{bits−1} levels per unit: scale up, round to integer, scale back.
    // `round_ties_even` (not `round`) — it pairs bit-exactly with CLIF's `nearest`.
    let b = ctx.inputs[1].max(1.0).min(24.0);
    let s = 2.0f32.powf(b - 1.0);
    out[0] = (ctx.inputs[0] * s).round_ties_even() / s;
}

fn tick_decim(ctx: &mut TickCtx, out: &mut [f32]) {
    // Sample-and-hold at `rate` Hz: advance a phase by rate/sr; on wrap, latch the input,
    // otherwise hold — `noiseh`'s latch with the input as the source.
    let rate = ctx.inputs[1].max(0.0);
    let phase = ctx.state[0] + rate / ctx.sr;
    let held = if phase >= 1.0 { ctx.inputs[0] } else { ctx.state[1] };
    ctx.state[0] = phase.rem_euclid(1.0);
    ctx.state[1] = held;
    out[0] = held;
}
