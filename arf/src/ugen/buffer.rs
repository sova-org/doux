//! Sample-memory generators: `delay`, `comb`, `allpass`. Each declares a power-of-two
//! buffer (the line) and a write head in one state slot. `comb` and `allpass` read the line
//! at a fractional position (`load_lerp`), so a modulated `time` glides instead of zipping.

use super::{Arity, Category, InputDescriptor, TickCtx, UGen, Unit, flush, signal};

pub(super) static UGENS: &[UGen] = &[
    // delay ( in time -- sig )  state: [write head]   buffer: the line
    UGen {
        name: "delay",
        category: Category::Delay,
        description: "Delay line — echoes the input `time` seconds later.",
        examples: &[
            "220 sine 0.2 * 0.3 delay out",
            "noise 0.02 *  e' 0.7 * +  0.25 delay  as e  out",
        ],
        arity: Arity::Fixed(2),
        inputs: &[
            signal("in"),
            InputDescriptor {
                name: "time",
                unit: Unit::Seconds,
                range: (0.0, 1.0),
                default: 0.1,
            },
        ],
        outputs: 1,
        state_slots: 1,
        buffer_len: 1 << 16,
        cost: 4,
        tick: tick_delay,
    },
    // comb ( in time fb -- sig )  state: [write head]   buffer: the line
    UGen {
        name: "comb",
        category: Category::Delay,
        description: "Feedback comb — an interpolated delay feeding itself back by `fb`; fb 0 is a clean modulatable delay (chorus, flanger).",
        examples: &[
            "8 impulse 0.05 0.7 comb 0.3 * out",
            "110 saw 0.15 *  0.3 sine 0.002 * 0.011 +  0 comb  0.5 * out",
        ],
        arity: Arity::Fixed(3),
        inputs: &[
            signal("in"),
            InputDescriptor {
                name: "time",
                unit: Unit::Seconds,
                range: (0.0, 1.0),
                default: 0.3,
            },
            InputDescriptor {
                name: "fb",
                unit: Unit::Ratio,
                range: (-0.95, 0.95),
                default: 0.7,
            },
        ],
        outputs: 1,
        state_slots: 1,
        buffer_len: 1 << 16,
        cost: 8,
        tick: tick_comb,
    },
    // allpass ( in time fb -- sig )  state: [write head]   buffer: the line
    UGen {
        name: "allpass",
        category: Category::Delay,
        description: "Schroeder allpass — flat magnitude, dense `time`-spaced echoes; chain a few after `comb`s for a reverb tank.",
        examples: &[
            "8 impulse 0.03 0.5 allpass 0.3 * out",
            "noise 0.2 *  0.013 0.6 allpass  0.011 0.6 allpass  0.4 * out",
        ],
        arity: Arity::Fixed(3),
        inputs: &[
            signal("in"),
            InputDescriptor {
                name: "time",
                unit: Unit::Seconds,
                range: (0.0, 1.0),
                default: 0.05,
            },
            InputDescriptor {
                name: "fb",
                unit: Unit::Ratio,
                range: (0.0, 0.95),
                default: 0.5,
            },
        ],
        outputs: 1,
        state_slots: 1,
        buffer_len: 1 << 16,
        cost: 8,
        tick: tick_allpass,
    },
];

/// Compile-time length (f32s, a power of two so the ticks can mask) for a delay-family
/// line at `sr`: sized to the `time` input when it is a literal, else to the input's
/// declared range max. The rows' fixed `buffer_len` never applies — this hook (reached via
/// [`super::sized_buffer_len`]) covers every anonymous instance, so a 10 ms literal comb
/// costs 1 KiB instead of the worst-case line, and the worst case itself tracks the actual
/// sample rate instead of assuming 48 kHz.
pub(super) fn line_len(sr: f32, consts: &[Option<f32>]) -> usize {
    // `time`'s declared range max (the rows agree); a modulated line gets the full range.
    const MAX_SECONDS: f32 = 1.0;
    // NaN-suppressing max/min, as the ticks clamp: a NaN literal collapses to 0.
    #[allow(clippy::manual_clamp)]
    let t = consts
        .get(1)
        .copied()
        .flatten()
        .unwrap_or(MAX_SECONDS)
        .max(0.0)
        .min(MAX_SECONDS);
    // +2: one for the write head, one so the interpolated read never wraps onto the head.
    (((t * sr).ceil() as usize) + 2).next_power_of_two()
}

#[allow(clippy::manual_clamp)] // max/min de-NaNs; `clamp` would propagate NaN (see `tick_comb`)
fn tick_delay(ctx: &mut TickCtx, out: &mut [f32]) {
    // A masked ring line: write the input at the head, read `time` seconds earlier
    // (truncated to whole samples, clamped within the line), then advance the head. The
    // buffer length is a power of two, so `& mask` keeps every index in range.
    let mask = ctx.buffer.len() - 1;
    let head = ctx.state[0] as usize;
    ctx.buffer[head & mask] = ctx.inputs[0];
    let delay = (ctx.inputs[1] * ctx.sr).max(0.0).min(mask as f32) as usize;
    out[0] = ctx.buffer[head.wrapping_sub(delay) & mask];
    ctx.state[0] = ((head + 1) & mask) as f32;
}

// Not `.clamp()`: `.max().min()` suppresses NaN (`clamp` propagates it), so a NaN time/fb
// collapses to the bound instead of latching into the line.
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
    ctx.buffer[head] = flush(ctx.inputs[0] + g * y);
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
    ctx.buffer[head] = flush(w);
    ctx.state[0] = ((head + 1) & mask) as f32;
    out[0] = r - g * w;
}
