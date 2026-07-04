//! Panners: `pan2`. Equal-power (constant-power) stereo panning via a square-root law, so the
//! summed power holds constant across the sweep; mono in, left/right out. A multi-output UGen:
//! one instance produces both channels.

use super::{signal, Arity, Category, InputDescriptor, TickCtx, UGen, Unit};

pub(super) static UGENS: &[UGen] = &[
    // pan2 ( in pos -- l r )   pos -1 = hard left .. +1 = hard right; equal-power, stateless
    UGen { name: "pan2", category: Category::Panner, description: "Equal-power stereo panner — mono in, L/R out; `pos` -1 (left) … +1 (right).",
           examples: &["440 sine 0.2 * 0 pan2 out", "440 sine 0.2 *  0.5 sine  pan2 out"], arity: Arity::Fixed(2),
           inputs: &[signal("in"), InputDescriptor { name: "pos", unit: Unit::Ratio, range: (-1.0, 1.0), default: 0.0 }],
           outputs: 2, state_slots: 0, buffer_len: 0, cost: 10, tick: tick_pan2 },
];

// `.max(-1).min(1)` not `.clamp()`: it suppresses NaN (`clamp` propagates it), so a NaN `pos`
// collapses to a bound instead of silencing both channels (the SVF core clamps the same way).
#[allow(clippy::manual_clamp)]
fn tick_pan2(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    // Clamp pos via NaN-suppressing max/min, then
    // gains = sqrt((1 ∓ pos)/2): equal power, summing to x² across the sweep.
    let pos = ctx.inputs[1].max(-1.0).min(1.0);
    let half_pos = 0.5 * pos;
    out[0] = x * (0.5 - half_pos).sqrt(); // left  = sqrt((1 - pos)/2)
    out[1] = x * (0.5 + half_pos).sqrt(); // right = sqrt((1 + pos)/2)
}
