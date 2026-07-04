//! Sample-memory generators: `delay`, `comb`, `allpass`. Each declares a power-of-two
//! buffer (the line) and a write head in one state slot. `comb` and `allpass` read the line
//! at a fractional position (`load_lerp`), so a modulated `time` glides instead of zipping.

use super::{signal, Arity, Category, InputDescriptor, Rate, TickCtx, UGen, Unit};

pub(super) static UGENS: &[UGen] = &[
    // delay ( in time -- sig )  state: [write head]   buffer: the line
    UGen { name: "delay", category: Category::Delay, description: "Delay line — echoes the input `time` seconds later.",
           examples: &["220 sine 0.2 * 0.3 delay out", "noise 0.02 *  e' 0.7 * +  0.25 delay  as e  out"], arity: Arity::Fixed(2),
           inputs: &[signal("in"), InputDescriptor { name: "time", unit: Unit::Seconds, range: (0.0, 1.0), default: 0.1, rate: Rate::Audio }],
           outputs: 1, state_slots: 1, buffer_len: 1 << 16, rate: Rate::Audio, cost: 4, tick: tick_delay },
    // comb ( in time fb -- sig )  state: [write head]   buffer: the line
    UGen { name: "comb", category: Category::Delay, description: "Feedback comb — an interpolated delay feeding itself back by `fb`; fb 0 is a clean modulatable delay (chorus, flanger).",
           examples: &["8 impulse 0.05 0.7 comb 0.3 * out", "110 saw 0.15 *  0.3 sine 0.002 * 0.011 +  0 comb  0.5 * out"], arity: Arity::Fixed(3),
           inputs: &[signal("in"),
                     InputDescriptor { name: "time", unit: Unit::Seconds, range: (0.0, 1.0), default: 0.3, rate: Rate::Audio },
                     InputDescriptor { name: "fb", unit: Unit::Ratio, range: (-0.95, 0.95), default: 0.7, rate: Rate::Audio }],
           outputs: 1, state_slots: 1, buffer_len: 1 << 16, rate: Rate::Audio, cost: 8, tick: tick_comb },
    // allpass ( in time fb -- sig )  state: [write head]   buffer: the line
    UGen { name: "allpass", category: Category::Delay, description: "Schroeder allpass — flat magnitude, dense `time`-spaced echoes; chain a few after `comb`s for a reverb tank.",
           examples: &["8 impulse 0.03 0.5 allpass 0.3 * out", "noise 0.2 *  0.013 0.6 allpass  0.011 0.6 allpass  0.4 * out"], arity: Arity::Fixed(3),
           inputs: &[signal("in"),
                     InputDescriptor { name: "time", unit: Unit::Seconds, range: (0.0, 1.0), default: 0.05, rate: Rate::Audio },
                     InputDescriptor { name: "fb", unit: Unit::Ratio, range: (0.0, 0.95), default: 0.5, rate: Rate::Audio }],
           outputs: 1, state_slots: 1, buffer_len: 1 << 16, rate: Rate::Audio, cost: 8, tick: tick_allpass },
];

fn tick_delay(ctx: &mut TickCtx, out: &mut [f32]) {
    // A masked ring line: write the input at the head, read `time` seconds earlier
    // (truncated to whole samples, clamped within the line), then advance the head. The
    // buffer length is a power of two, so `& mask` keeps every index in range.
    let mask = ctx.buffer.len() - 1;
    let head = ctx.state[0] as usize;
    ctx.buffer[head & mask] = ctx.inputs[0];
    let delay = (ctx.inputs[1] * ctx.sr).clamp(0.0, mask as f32) as usize;
    out[0] = ctx.buffer[head.wrapping_sub(delay) & mask];
    ctx.state[0] = ((head + 1) & mask) as f32;
}

// Not `.clamp()`: the time and fb bounds mirror the JIT's NaN-suppressing max/min shims.
#[allow(clippy::manual_clamp)]
fn tick_comb(ctx: &mut TickCtx, out: &mut [f32]) {
    // Read the line `time` seconds back (interpolated, read-before-write so the tap can
    // reach the full line), then write input + fb·read at the head. The delay is floored at
    // one sample — the shortest loop a single-write line supports — and the feedback gain
    // is bounded inside ±1 so the loop cannot run away.
    let mask = ctx.buffer.len() - 1;
    let head = ctx.state[0] as usize & mask;
    let d = (ctx.inputs[1] * ctx.sr).max(1.0).min(mask as f32);
    let pos = head as f32 + ctx.buffer.len() as f32 - d;
    let y = ctx.load_lerp(pos);
    let g = ctx.inputs[2].max(-0.999).min(0.999);
    ctx.buffer[head] = ctx.inputs[0] + g * y;
    ctx.state[0] = ((head + 1) & mask) as f32;
    out[0] = y;
}

// Not `.clamp()`: same NaN-suppressing max/min mirroring as `tick_comb`.
#[allow(clippy::manual_clamp)]
fn tick_allpass(ctx: &mut TickCtx, out: &mut [f32]) {
    // Schroeder allpass around the same interpolated line as `comb`:
    // w = in + g·r; out = r − g·w; the line stores w. Unity magnitude at every frequency.
    let mask = ctx.buffer.len() - 1;
    let head = ctx.state[0] as usize & mask;
    let d = (ctx.inputs[1] * ctx.sr).max(1.0).min(mask as f32);
    let pos = head as f32 + ctx.buffer.len() as f32 - d;
    let r = ctx.load_lerp(pos);
    let g = ctx.inputs[2].max(0.0).min(0.999);
    let w = ctx.inputs[0] + g * r;
    ctx.buffer[head] = w;
    ctx.state[0] = ((head + 1) & mask) as f32;
    out[0] = r - g * w;
}
