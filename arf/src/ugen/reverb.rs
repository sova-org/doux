//! Reverb: `verb`, a Freeverb-style mono tank in one word — 8 parallel damped feedback combs
//! summed into 4 series Schroeder allpasses.
//!
//! All twelve delay lines live in *one* op buffer (`1 << 17` f32s, 512 KiB), partitioned at
//! fixed compile-time offsets: 8 comb regions of [`COMB_CAP`] samples, then 4 allpass regions
//! of [`AP_CAP`]. The classic Freeverb tunings are carried as *seconds* ([`COMB_T`]/[`AP_T`])
//! and scaled by the running sample rate, clamped into each region — so the tank keeps its
//! sound at any rate and an absurd rate shortens the tail instead of corrupting a neighbor
//! region. Each line keeps its own write head in a state slot (a head wraps `% len`, and the
//! lengths are not powers of two — a shared counter would click on its wrap); the comb damping
//! filters add eight more slots, 20 in all, and everything migrates across a hot-swap like any
//! other UGen state (the buffer is donated whole).

use super::{signal, Arity, Category, InputDescriptor, Rate, TickCtx, UGen, Unit};

/// The classic Freeverb comb tunings (sample counts at 44.1 kHz) as seconds.
const COMB_T: [f32; 8] = [
    1116.0 / 44100.0, 1188.0 / 44100.0, 1277.0 / 44100.0, 1356.0 / 44100.0,
    1422.0 / 44100.0, 1491.0 / 44100.0, 1557.0 / 44100.0, 1617.0 / 44100.0,
];
/// The classic Freeverb allpass tunings (sample counts at 44.1 kHz) as seconds.
const AP_T: [f32; 4] = [556.0 / 44100.0, 441.0 / 44100.0, 341.0 / 44100.0, 225.0 / 44100.0];

/// Per-comb region capacity: the longest comb at 192 kHz is ⌈1617/44100·192000⌉ = 7040 ≤ 8192.
const COMB_CAP: usize = 8192;
/// Per-allpass region capacity: the longest allpass at 192 kHz is 2421 ≤ 4096.
const AP_CAP: usize = 4096;
/// Region offsets inside the one partitioned buffer: combs first, then allpasses.
const fn comb_off(k: usize) -> usize {
    k * COMB_CAP
}
const fn ap_off(j: usize) -> usize {
    8 * COMB_CAP + j * AP_CAP
}
/// Total partitioned extent is 8·8192 + 4·4096 = 81 920, rounded up to a power of two.
const BUF_LEN: usize = 1 << 17;

pub(super) static UGENS: &[UGen] = &[
    // verb ( in mix room damp -- sig )  state: [8 comb heads, 8 damp memories, 4 ap heads]
    UGen { name: "verb", category: Category::Delay, description: "Freeverb-style mono reverb — 8 damped feedback combs into 4 series allpasses; `mix` blends dry→wet.",
           examples: &["2 impulse 0.05 perc 440 sine * 0.3 *  0.4 0.9 0.5 verb  0.5 * out", "110 saw 0.2 *  0.3 0.7 0.5 verb  out", "4 impulse 0.02 perc noise *  0.6 0.95 0.2 verb  0.4 * out"], arity: Arity::Fixed(4),
           inputs: &[signal("in"),
                     InputDescriptor { name: "mix", unit: Unit::Ratio, range: (0.0, 1.0), default: 0.3, rate: Rate::Audio },
                     InputDescriptor { name: "room", unit: Unit::Ratio, range: (0.0, 1.0), default: 0.7, rate: Rate::Audio },
                     InputDescriptor { name: "damp", unit: Unit::Ratio, range: (0.0, 1.0), default: 0.5, rate: Rate::Audio }],
           outputs: 1, state_slots: 20, buffer_len: BUF_LEN, rate: Rate::Audio, cost: 80, tick: tick_verb },
];

// Not `.clamp()`: the mix/room/damp bounds mirror the JIT's NaN-suppressing max/min shims.
#[allow(clippy::manual_clamp, clippy::needless_range_loop)]
fn tick_verb(ctx: &mut TickCtx, out: &mut [f32]) {
    // The Freeverb topology with its classic scalings: input gain 0.015 into 8 parallel
    // combs (feedback `room` mapped onto 0.7..0.98, one-pole damping in the loop), the sum
    // through 4 series allpasses (g = 0.5), wet makeup ×3, equal blend with the dry input.
    // Each line: read-before-write at the head (delay = len), so the tap reaches the full
    // line. Heads stay `< len ≤ CAP` (the `%` below), so `off + head` never leaves a region.
    let x = ctx.inputs[0];
    let mix = ctx.inputs[1].max(0.0).min(1.0);
    let room = ctx.inputs[2].max(0.0).min(1.0) * 0.28 + 0.7;
    let damp = ctx.inputs[3].max(0.0).min(1.0) * 0.4;
    let input = x * 0.015;
    let mut acc = 0.0;
    for k in 0..8 {
        let len = (COMB_T[k] * ctx.sr).max(1.0).min(COMB_CAP as f32) as usize;
        let head = (ctx.state[k] as usize) % len;
        let idx = comb_off(k) + head;
        let y = ctx.buffer[idx];
        let f = y * (1.0 - damp) + ctx.state[8 + k] * damp; // one-pole damping in the loop
        ctx.state[8 + k] = f;
        ctx.buffer[idx] = input + room * f;
        ctx.state[k] = ((head + 1) % len) as f32;
        acc += y; // left-fold in comb order — load-bearing, `emit_verb` mirrors it
    }
    let mut s = acc;
    for j in 0..4 {
        let len = (AP_T[j] * ctx.sr).max(1.0).min(AP_CAP as f32) as usize;
        let head = (ctx.state[16 + j] as usize) % len;
        let idx = ap_off(j) + head;
        let r = ctx.buffer[idx];
        let w = s + 0.5 * r;
        ctx.buffer[idx] = w;
        ctx.state[16 + j] = ((head + 1) % len) as f32;
        s = r - 0.5 * w;
    }
    out[0] = (1.0 - mix) * x + mix * (s * 3.0);
}
