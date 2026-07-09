//! Reverb: `verb`, a Freeverb-style mono tank in one word — 8 parallel damped feedback combs
//! summed into 4 series Schroeder allpasses.
//!
//! All twelve delay lines live in *one* op buffer, partitioned at fixed offsets derived from
//! the region capacity `C` = [`comb_cap`]: 8 comb regions of `C` samples, then 4 allpass
//! regions of `C/2`, so the tank is `10·C` and a tick can recover `C` from the buffer length
//! alone. The compiler sizes the buffer to the program's sample rate via [`tank_len`]
//! (reached through [`super::sized_buffer_len`]) — ~80 KiB at 48 kHz instead of a fixed
//! 192 kHz worst case. The classic Freeverb tunings are carried as *seconds*
//! ([`COMB_T`]/[`AP_T`]) and scaled by the running sample rate, clamped into each region — so
//! the tank keeps its sound at any rate. Each line keeps its own write head in a state slot
//! (heads stay `< len` by construction — every store re-wraps — so advancing is a
//! compare-and-reset, no `%`; the lengths are not powers of two and a shared counter would
//! click on its wrap); the comb damping filters add eight more slots, 20 in all.

use super::{flush, signal, Arity, Category, InputDescriptor, TickCtx, UGen, Unit};

/// The classic Freeverb comb tunings (sample counts at 44.1 kHz) as seconds, longest last.
const COMB_T: [f32; 8] = [
    1116.0 / 44100.0, 1188.0 / 44100.0, 1277.0 / 44100.0, 1356.0 / 44100.0,
    1422.0 / 44100.0, 1491.0 / 44100.0, 1557.0 / 44100.0, 1617.0 / 44100.0,
];
/// The classic Freeverb allpass tunings (sample counts at 44.1 kHz) as seconds.
const AP_T: [f32; 4] = [556.0 / 44100.0, 441.0 / 44100.0, 341.0 / 44100.0, 225.0 / 44100.0];

/// Per-comb region capacity at `sr`: the longest comb tuning, ceiled, at least one sample,
/// rounded to a power of two so `C/2` stays exact.
fn comb_cap(sr: f32) -> usize {
    (((COMB_T[7] * sr).ceil() as usize).max(1)).next_power_of_two()
}

/// Compile-time tank size at `sr`: 8 comb regions of `C` plus 4 allpass regions of `C/2`,
/// i.e. `10·C`. The longest allpass tuning is 556/1617 ≈ 0.34 of the longest comb, so `C/2`
/// always fits it.
pub(super) fn tank_len(sr: f32) -> usize {
    10 * comb_cap(sr)
}

pub(super) static UGENS: &[UGen] = &[
    // verb ( in mix room damp -- sig )  state: [8 comb heads, 8 damp memories, 4 ap heads]
    // The row's buffer_len is the documented 192 kHz worst case; the compiler supersedes it
    // per program via `tank_len` (see `sized_buffer_len`).
    UGen { name: "verb", category: Category::Delay, description: "Freeverb-style mono reverb — 8 damped feedback combs into 4 series allpasses; `mix` blends dry→wet.",
           examples: &["2 impulse 0.05 perc 440 sine * 0.3 *  0.4 0.9 0.5 verb  0.5 * out", "110 saw 0.2 *  0.3 0.7 0.5 verb  out", "4 impulse 0.02 perc noise *  0.6 0.95 0.2 verb  0.4 * out"], arity: Arity::Fixed(4),
           inputs: &[signal("in"),
                     InputDescriptor { name: "mix", unit: Unit::Ratio, range: (0.0, 1.0), default: 0.3 },
                     InputDescriptor { name: "room", unit: Unit::Ratio, range: (0.0, 1.0), default: 0.7 },
                     InputDescriptor { name: "damp", unit: Unit::Ratio, range: (0.0, 1.0), default: 0.5 }],
           outputs: 1, state_slots: 20, buffer_len: 1 << 17, cost: 80, tick: tick_verb },
];

// Not `.clamp()`: `.max().min()` suppresses NaN (`clamp` propagates it), so a NaN mix/room/damp
// collapses to a bound instead of latching NaN into the tank.
#[allow(clippy::manual_clamp, clippy::needless_range_loop)]
fn tick_verb(ctx: &mut TickCtx, out: &mut [f32]) {
    // The Freeverb topology with its classic scalings: input gain 0.015 into 8 parallel
    // combs (feedback `room` mapped onto 0.7..0.98, one-pole damping in the loop), the sum
    // through 4 series allpasses (g = 0.5), wet makeup ×3, equal blend with the dry input.
    // Each line: read-before-write at the head (delay = len), so the tap reaches the full
    // line. Heads stay `< len ≤ cap` (every store below re-wraps), so `off + head` never
    // leaves a region and the advance needs no `%`.
    let x = ctx.inputs[0];
    let mix = ctx.inputs[1].max(0.0).min(1.0);
    let room = ctx.inputs[2].max(0.0).min(1.0) * 0.28 + 0.7;
    let damp = ctx.inputs[3].max(0.0).min(1.0) * 0.4;
    // Recover the compile-time region capacity from the tank size (see the module doc).
    let c = ctx.buffer.len() / 10;
    let input = x * 0.015;
    let mut acc = 0.0;
    for k in 0..8 {
        let len = (COMB_T[k] * ctx.sr).max(1.0).min(c as f32) as usize;
        let head = ctx.state[k] as usize;
        let idx = k * c + head;
        let y = ctx.buffer[idx];
        let f = flush(y * (1.0 - damp) + ctx.state[8 + k] * damp); // one-pole damping in the loop
        ctx.state[8 + k] = f;
        ctx.buffer[idx] = flush(input + room * f);
        let next = head + 1;
        ctx.state[k] = (if next >= len { 0 } else { next }) as f32;
        acc += y; // left-fold in comb order — load-bearing for f32 determinism
    }
    let mut s = acc;
    let ap_cap = c / 2;
    for j in 0..4 {
        let len = (AP_T[j] * ctx.sr).max(1.0).min(ap_cap as f32) as usize;
        let head = ctx.state[16 + j] as usize;
        let idx = 8 * c + j * ap_cap + head;
        let r = ctx.buffer[idx];
        let w = s + 0.5 * r;
        ctx.buffer[idx] = flush(w);
        let next = head + 1;
        ctx.state[16 + j] = (if next >= len { 0 } else { next }) as f32;
        s = r - 0.5 * w;
    }
    out[0] = (1.0 - mix) * x + mix * (s * 3.0);
}
