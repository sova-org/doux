//! Clocks, triggers, and envelopes: `impulse`, `trig`, `perc`, `seq`, `line`, `xline`, `phasor`,
//! `ar`, `adsr`, `latch`.
//!
//! # The trigger / gate convention (modular)
//!
//! Time and control flow through ordinary signals, patched like a modular synth:
//! - a **trigger** is a *rising edge* — a signal crossing from `≤ 0` to `> 0`. `trig` detects one;
//!   `impulse` and a `phasor`-wrap emit them; a scheduled event or a manual `1` does too.
//! - a **gate** is a *level* — `> 0` means held. The control plane's `gate` lane is one.
//!
//! A triggerable envelope (`line`, `ar`) captures its start time on a *rising edge*, so any trigger
//! signal re-arms it — patch in an `impulse`, a clock, a manual bang. A gated envelope (`adsr`)
//! follows the *level*: it sustains while the gate is high and releases when it drops. `perc` and
//! `seq` react to a trigger being present (`> 0`). This is the one rule for "trig / retrig easily".
//!
//! # Re-eval policy
//!
//! Envelope/clock state (a phase, a step, a captured start time) is ordinary `state_slots`,
//! owned by the instance's Vm: a playing instance keeps its Vm whole across a re-edit, so its
//! envelopes continue; a fresh instance starts from fresh state. Musical retriggering is done
//! with trigger *signals* (above), not by re-evaluating the text.

use super::{
    Arity, Category, InputDescriptor, ListShape, TickCtx, UGen, Unit, flush, signal, wrap01,
};
use crate::fastmath::powf;

pub(super) static UGENS: &[UGen] = &[
    // impulse ( rate -- trig )   state: [phase]   one-sample 1 at each period end, else 0
    UGen {
        name: "impulse",
        category: Category::Trigger,
        description: "Clock — fires a one-sample impulse at `rate` Hz.",
        examples: &[
            "4 impulse 0.3 * out",
            "8 impulse 0.05 perc 440 sine * 0.3 * out",
        ],
        arity: Arity::Fixed(1),
        inputs: &[InputDescriptor {
            name: "rate",
            unit: Unit::Hz,
            range: (0.0, 20_000.0),
            default: 1.0,
        }],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 5,
        tick: tick_impulse,
    },
    // trig ( in -- trig )   state: [prev]   1 on a rising edge (≤0 → >0), else 0
    UGen {
        name: "trig",
        category: Category::Trigger,
        description: "Rising-edge detector — fires a one-sample impulse when the input crosses from ≤0 to >0.",
        examples: &[
            "2 sine trig 0.1 perc 440 sine * 0.3 * out",
            "4 impulse [ 1 0 1 1 ] seq trig 0.05 perc noise * 0.3 * out",
        ],
        arity: Arity::Fixed(1),
        inputs: &[signal("in")],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 2,
        tick: tick_trig,
    },
    // perc ( trig time -- env )   state: [phase]   linear decay 1→0 over `time` s, reset on trig
    UGen {
        name: "perc",
        category: Category::Envelope,
        description: "Percussive envelope — linear decay 1→0 over `time` seconds, reset by a trigger.",
        examples: &[
            "4 impulse 0.1 perc 440 sine * 0.3 * out",
            "2 impulse 0.3 perc noise * 0.3 * out",
        ],
        arity: Arity::Fixed(2),
        inputs: &[
            signal("trig"),
            InputDescriptor {
                name: "time",
                unit: Unit::Seconds,
                range: (0.0, 10.0),
                default: 0.1,
            },
        ],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 4,
        tick: tick_perc,
    },
    // seq ( trig [v0 v1 …] -- val )   state: [step]   step through the value-list on each trig.
    // Built by a front-end's `VariadicLed` arm: inputs[0] is the trigger, inputs[1..] the
    // values, so it is variadic. The read holds the current value between triggers.
    UGen {
        name: "seq",
        category: Category::Trigger,
        description: "Step sequencer — holds a value from the list, advancing one step on each trigger.",
        examples: &[
            "4 impulse [ 220 330 440 550 ] seq sine 0.2 * out",
            "8 impulse [ 0 5 7 12 ] seq 60 + mtof sine 0.2 * out",
        ],
        arity: Arity::VariadicLed {
            shape: ListShape::Any,
        },
        inputs: &[signal("trig")],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 3,
        tick: tick_seq,
    },
    // line ( trig start end dur -- y )   state: [start, prev, armed]   linear travel start→end,
    // retriggered by an edge; rests at `start` until first triggered, holds `end` after
    UGen {
        name: "line",
        category: Category::Envelope,
        description: "Linear line from `start` to `end` over `dur` seconds, (re)started by a trigger; holds `end`, rests at `start` until first triggered.",
        examples: &[
            "1 trig 110 550 0.5 line sine 0.2 * out",
            "2 impulse 0 1 0.3 line 220 sine * 0.3 * out",
        ],
        arity: Arity::Fixed(4),
        inputs: &[
            signal("trig"),
            signal("start"),
            signal("end"),
            InputDescriptor {
                name: "dur",
                unit: Unit::Seconds,
                range: (0.0, 60.0),
                default: 1.0,
            },
        ],
        outputs: 1,
        state_slots: 3,
        buffer_len: 0,
        cost: 8,
        tick: tick_line,
    },
    // xline ( trig start end dur -- y )   state: [start, prev, armed]   exponential travel
    // start→end (same sign, nonzero) — the natural curve for pitch and cutoff drops
    UGen {
        name: "xline",
        category: Category::Envelope,
        description: "Exponential line from `start` to `end` over `dur` seconds (same sign, nonzero), (re)started by a trigger; holds `end`.",
        examples: &[
            "2 impulse 400 50 0.08 xline sine 0.3 * out",
            "1 trig 880 110 0.7 xline sine 0.2 * out",
        ],
        arity: Arity::Fixed(4),
        inputs: &[
            signal("trig"),
            signal("start"),
            signal("end"),
            InputDescriptor {
                name: "dur",
                unit: Unit::Seconds,
                range: (0.0, 60.0),
                default: 1.0,
            },
        ],
        outputs: 1,
        state_slots: 3,
        buffer_len: 0,
        cost: 16,
        tick: tick_xline,
    },
    // phasor ( freq -- phase )   state: [phase]   precise 0→1 ramp at `freq` Hz
    UGen {
        name: "phasor",
        category: Category::Oscillator,
        description: "Ramp oscillator — a 0→1 phase ramp at `freq` Hz; the precise phase source for saws, LFOs, and tempo sync.",
        examples: &[
            "1 phasor 2 * 1 - 0.2 * out",
            "0.25 phasor 400 * 100 + sine 0.2 * out",
        ],
        arity: Arity::Fixed(1),
        inputs: &[InputDescriptor {
            name: "freq",
            unit: Unit::Hz,
            range: (0.0, 20_000.0),
            default: 1.0,
        }],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 5,
        tick: tick_phasor,
    },
    // clock ( rate -- trig )   state: [beat, started]   drift-free clock locked to the global `now`
    UGen {
        name: "clock",
        category: Category::Trigger,
        description: "Clock locked to the global sample clock — fires at `rate` Hz with no drift; clocks at related rates stay sample-exact in sync.",
        examples: &[
            "4 clock 0.05 perc 440 sine * 0.3 * out",
            "[ 3 clock 4 clock ] 0.05 perc 330 sine * 0.2 * out",
        ],
        arity: Arity::Fixed(1),
        inputs: &[InputDescriptor {
            name: "rate",
            unit: Unit::Hz,
            range: (0.0, 20_000.0),
            default: 1.0,
        }],
        outputs: 1,
        state_slots: 2,
        buffer_len: 0,
        cost: 6,
        tick: tick_clock,
    },
    // ar ( trig attack release -- y )   state: [start, prev, armed, captured]   triggered
    // attack-release; a retrigger rises from the captured current level (click-free)
    UGen {
        name: "ar",
        category: Category::Envelope,
        description: "Attack-release envelope — rises to 1 over `attack` then falls to 0 over `release`, (re)started by a trigger; a retrigger rises from the current level (click-free).",
        examples: &[
            "2 impulse 0.01 0.3 ar 440 sine * 0.3 * out",
            "1 impulse 0.2 0.5 ar 110 saw 800 lpf * 0.3 * out",
        ],
        arity: Arity::Fixed(3),
        inputs: &[
            signal("trig"),
            InputDescriptor {
                name: "attack",
                unit: Unit::Seconds,
                range: (0.0, 10.0),
                default: 0.01,
            },
            InputDescriptor {
                name: "release",
                unit: Unit::Seconds,
                range: (0.0, 10.0),
                default: 0.2,
            },
        ],
        outputs: 1,
        state_slots: 4,
        buffer_len: 0,
        cost: 8,
        tick: tick_ar,
    },
    // adsr ( gate attack decay sustain release -- y )   state: [prev, atk_start, rel_start,
    // rel_level, atk_level]   gated; a re-gate rises from the captured release value (click-free)
    UGen {
        name: "adsr",
        category: Category::Envelope,
        description: "Gated ADSR envelope — attack/decay to `sustain` while the gate is held, release to 0 when it drops; a re-gate rises from the current level (click-free).",
        examples: &[
            "2 sine 0 > 0.01 0.1 0.7 0.3 adsr 220 saw * 0.3 * out",
            "gate 0.05 0.2 0.6 0.4 adsr notefreq sine * 0.3 * out",
        ],
        arity: Arity::Fixed(5),
        inputs: &[
            signal("gate"),
            InputDescriptor {
                name: "attack",
                unit: Unit::Seconds,
                range: (0.0, 10.0),
                default: 0.01,
            },
            InputDescriptor {
                name: "decay",
                unit: Unit::Seconds,
                range: (0.0, 10.0),
                default: 0.1,
            },
            InputDescriptor {
                name: "sustain",
                unit: Unit::Amplitude,
                range: (0.0, 1.0),
                default: 0.7,
            },
            InputDescriptor {
                name: "release",
                unit: Unit::Seconds,
                range: (0.0, 10.0),
                default: 0.3,
            },
        ],
        outputs: 1,
        state_slots: 5,
        buffer_len: 0,
        cost: 12,
        tick: tick_adsr,
    },
    // latch ( in trig -- held )   state: [held, prev]   sample & hold on a rising edge
    UGen {
        name: "latch",
        category: Category::Trigger,
        description: "Sample & hold — captures the input on each rising trigger edge and holds it.",
        examples: &[
            "noise 8 impulse latch 400 * 500 + sine 0.2 * out",
            "noise 6 impulse latch 0.5 * 0.5 + 440 * sine 0.2 * out",
        ],
        arity: Arity::Fixed(2),
        inputs: &[signal("in"), signal("trig")],
        outputs: 1,
        state_slots: 2,
        buffer_len: 0,
        cost: 3,
        tick: tick_latch,
    },
    // decay ( in time -- env )   state: [y]   SC Decay: leaky integrator, 60 dB exp decay
    UGen {
        name: "decay",
        category: Category::Envelope,
        description: "Exponential decay — each input impulse rings down 60 dB over `time` seconds; overlapping hits sum (SC Decay).",
        examples: &[
            "4 impulse 0.3 decay 440 sine * 0.3 * out",
            "8 impulse 0.1 decay noise * 0.3 * out",
        ],
        arity: Arity::Fixed(2),
        inputs: &[
            signal("in"),
            InputDescriptor {
                name: "time",
                unit: Unit::Seconds,
                range: (0.0, 10.0),
                default: 0.3,
            },
        ],
        outputs: 1,
        state_slots: 3,
        buffer_len: 0,
        cost: 12,
        tick: tick_decay,
    },
    // linseg ( trig [l0 t1 l1 t2 l2 …] -- y )   state: [start, prev, armed]   variadic breakpoint
    // envelope, built by a front-end's `VariadicLed` arm: input 0 is the trigger, inputs 1..
    // the flattened level/time list (start level l0, then (time, level) pairs). Clock-relative.
    UGen {
        name: "linseg",
        category: Category::Envelope,
        description: "Multi-segment breakpoint envelope — ramps through a `[ l0 t1 l1 … ]` level/time list on each trigger, then holds the last level (the general form of perc/ar).",
        examples: &[
            "4 impulse [ 0 0.005 1 0.2 0 ] linseg 440 sine * 0.3 * out",
            "1 trig [ 0 0.5 1 0.5 0.3 ] linseg 200 * 200 + sine 0.2 * out",
        ],
        arity: Arity::VariadicLed {
            shape: ListShape::OddAtLeastThree,
        },
        inputs: &[signal("trig")],
        outputs: 1,
        state_slots: 3,
        buffer_len: 0,
        cost: 10,
        tick: tick_linseg,
    },
];

/// Clamp to [0, 1] with `.max().min()`, not `f32::clamp`: max/min suppress NaN whereas `clamp`
/// propagates it, so a NaN input collapses to a bound instead of leaking into the envelope.
/// The single clamp used by every envelope tick here.
#[allow(clippy::manual_clamp)]
fn clamp01(x: f32) -> f32 {
    x.max(0.0).min(1.0)
}

fn tick_impulse(ctx: &mut TickCtx, out: &mut [f32]) {
    let inc = ctx.inputs[0].max(0.0) / ctx.sr; // `.max` de-NaNs, like `tick_clock`
    let p = ctx.state[0] + inc;
    out[0] = if p >= 1.0 { 1.0 } else { 0.0 }; // fire when the phase crosses a period
    ctx.state[0] = wrap01(p);
}

fn tick_trig(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    out[0] = if ctx.state[0] <= 0.0 && x > 0.0 {
        1.0
    } else {
        0.0
    };
    ctx.state[0] = x;
}

fn tick_perc(ctx: &mut TickCtx, out: &mut [f32]) {
    // Linear decay from 1 to 0 over `time` seconds; a trigger resets the phase to 0.
    // Clamp `time` to be non-negative (like `lag`/`delay`): a negative time would make the
    // increment negative and run the envelope away to a huge sustained gain. A non-positive
    // time gives 1/(0·sr) = +inf, so the phase saturates to 1 and the envelope finishes at once.
    let phase = if ctx.inputs[0] > 0.0 {
        0.0
    } else {
        (ctx.state[0] + 1.0 / (ctx.inputs[1].max(0.0) * ctx.sr)).min(1.0)
    };
    ctx.state[0] = phase;
    out[0] = 1.0 - phase;
}

fn tick_seq(ctx: &mut TickCtx, out: &mut [f32]) {
    let n = (ctx.inputs.len() - 1) as f32; // number of values (inputs[0] is the trigger)
    let step = ctx.state[0];
    out[0] = ctx.inputs[1 + step as usize]; // hold the current value (read before advancing)
    let next = step + 1.0;
    let wrapped = if next >= n { 0.0 } else { next };
    ctx.state[0] = if ctx.inputs[0] > 0.0 { wrapped } else { step };
}

/// The shared edge-triggered ramp core behind `line`/`xline`: a rising edge (prev ≤ 0 < trig)
/// captures the start time as `now`; the return is the 0→1 progress over `dur` seconds, held at
/// 1 after, and pinned to 0 until the first trigger (`armed`), so it never ramps from the
/// clock's origin. A pure function of `now - start`, so it is phase-deterministic.
/// State: [start, prev, armed]; `dur` is inputs[3].
fn line_progress(ctx: &mut TickCtx) -> f32 {
    let trig = ctx.inputs[0];
    let edge = ctx.state[1] <= 0.0 && trig > 0.0;
    let start = if edge { ctx.now } else { ctx.state[0] };
    let armed = if edge { 1.0 } else { ctx.state[2] };
    // At least one sample of ramp: a non-positive `dur` is an instant jump, never a divide by
    // zero (mirrors `perc`'s non-negative `time` guard). `now - start` is wrapped into the window.
    let denom = (ctx.inputs[3].max(0.0) * ctx.sr).max(1.0);
    let ramp = clamp01(crate::ir::now_wrap(ctx.now - start) / denom);
    ctx.state[0] = start;
    ctx.state[1] = trig;
    ctx.state[2] = armed;
    armed * ramp
}

fn tick_line(ctx: &mut TickCtx, out: &mut [f32]) {
    // Linear travel start→end by the shared progress: unarmed (progress 0) rests at `start`.
    let t = line_progress(ctx);
    out[0] = ctx.inputs[1] + (ctx.inputs[2] - ctx.inputs[1]) * t;
}

fn tick_xline(ctx: &mut TickCtx, out: &mut [f32]) {
    // Exponential travel: start·(end/start)^t. The ratio's magnitude is clamped away from
    // zero and its sign dropped, so a zero or sign-mixed pair follows `start`'s sign toward
    // |end| instead of going NaN — with the documented same-sign, nonzero args it is exact.
    let t = line_progress(ctx);
    let start = ctx.inputs[1];
    let end = ctx.inputs[2];
    let denom = if start == 0.0 { 1e-6 } else { start };
    let ratio = (end / denom).abs().max(1e-12);
    out[0] = start * powf(ratio, t);
}

#[allow(clippy::manual_clamp)] // max/min de-NaNs; `clamp` would propagate NaN
fn tick_phasor(ctx: &mut TickCtx, out: &mut [f32]) {
    // A 0→1 ramp at `freq` Hz: output the phase, then advance and wrap. Phase *accumulation* (not
    // a `now` read) keeps it precise for any frequency over unbounded runtime — `now * freq` would
    // exceed the f32 mantissa once the phase is large, so the modular reduction would lose bits.
    // Two phasors at the same rate stay sample-locked.
    let phase = ctx.state[0];
    out[0] = phase;
    // de-NaN and bound |inc| ≤ 1 (wrap01 handles reverse; |inc| > 1 aliases anyway).
    let inc = (ctx.inputs[0] / ctx.sr).max(-1.0).min(1.0);
    ctx.state[0] = wrap01(phase + inc);
}

fn tick_clock(ctx: &mut TickCtx, out: &mut [f32]) {
    // A clock locked to the shared global `now`, not a private accumulator: fire a one-sample
    // trigger each time the beat index `floor(now / period)` advances, plus once at the very start.
    // Because every `clock` reads the same `now`, clocks at *any* related rates fire on exactly the
    // same samples (3-against-4, 7-tuplets, dotted, long or short) — sample-exact lock, zero drift,
    // and the period is exactly `sr/rate` (unlike `impulse`, which overshoots and slowly drifts).
    // `rate` is an ordinary input, so it can be tempo-relative (`bps 4 *`) or even modulated.
    let period = ctx.sr / ctx.inputs[0].max(0.0); // rate ≤ 0 → +inf period → only the start pulse
    let beat = (ctx.now / period).floor();
    let started = ctx.state[1];
    let fire = started == 0.0 || beat > ctx.state[0];
    ctx.state[0] = beat;
    ctx.state[1] = 1.0;
    out[0] = if fire { 1.0 } else { 0.0 };
}

fn tick_ar(ctx: &mut TickCtx, out: &mut [f32]) {
    // Triggered attack-release: a rising edge captures the start; the level rises to 1 over
    // `attack` seconds then falls to 0 over `release`, resting at 0 before the first trigger and
    // after the tail. A retrigger mid-flight is click-free: the edge also captures the level the
    // envelope holds at that instant, and the attack rises captured→1 from there instead of
    // snapping back to 0. `now - start` (bounded by the envelope length) keeps it exact.
    let trig = ctx.inputs[0];
    let edge = ctx.state[1] <= 0.0 && trig > 0.0;
    let a = (ctx.inputs[1].max(0.0) * ctx.sr).max(1.0);
    let r = (ctx.inputs[2].max(0.0) * ctx.sr).max(1.0);
    // With cap = 0 (never retriggered mid-flight) the attack `cap + (1−cap)·e/a` is exactly `e/a`.
    let shape = |e: f32, cap: f32| {
        if e < a {
            cap + (1.0 - cap) * (e / a) // attack: rise captured→1
        } else if e < a + r {
            1.0 - (e - a) / r // release: fall 1→0
        } else {
            0.0 // finished
        }
    };
    // The level under the old state — what a retrigger must rise from.
    let e_old = crate::ir::now_wrap(ctx.now - ctx.state[0]);
    let level_old = ctx.state[2] * clamp01(shape(e_old, ctx.state[3]));
    let start = if edge { ctx.now } else { ctx.state[0] };
    let armed = if edge { 1.0 } else { ctx.state[2] };
    let captured = if edge { level_old } else { ctx.state[3] };
    let e = crate::ir::now_wrap(ctx.now - start);
    let level = shape(e, captured);
    ctx.state[0] = start;
    ctx.state[1] = trig;
    ctx.state[2] = armed;
    ctx.state[3] = captured;
    out[0] = armed * clamp01(level);
}

fn tick_adsr(ctx: &mut TickCtx, out: &mut [f32]) {
    // Gated ADSR. While the gate is held: rise to 1 over `attack`, decay to `sustain` over
    // `decay`, then hold `sustain`. When the gate drops: release from the level it had to 0 over
    // `release`. Both edges capture the live level so it never clicks: the falling edge captures
    // the release start level (mid-attack included), and the rising edge captures the current
    // release value (`atk_level`) so a re-gate rises from there instead of snapping back to 0
    // (a rising edge implies the gate was low, so the envelope was releasing or at rest).
    let gate = ctx.inputs[0];
    let a = (ctx.inputs[1].max(0.0) * ctx.sr).max(1.0);
    let d = (ctx.inputs[2].max(0.0) * ctx.sr).max(1.0);
    let sus = clamp01(ctx.inputs[3]);
    let r = (ctx.inputs[4].max(0.0) * ctx.sr).max(1.0);
    let prev = ctx.state[0];
    let rising = prev <= 0.0 && gate > 0.0;
    let falling = prev > 0.0 && gate <= 0.0;
    // The release value under the old state — what a re-gate rises from (the clamp01 also
    // de-NaNs a poisoned slot).
    let er_old = crate::ir::now_wrap(ctx.now - ctx.state[2]);
    let rel_val_old = clamp01(ctx.state[3] * (1.0 - er_old / r).max(0.0));
    let atk_start = if rising { ctx.now } else { ctx.state[1] };
    let atk_level = if rising { rel_val_old } else { ctx.state[4] };
    // The attack→decay→sustain level as if the gate were still held. With atk_level = 0 (a
    // fresh attack) the rise `lvl + (1−lvl)·ea/a` is exactly `ea/a`.
    let ea = crate::ir::now_wrap(ctx.now - atk_start);
    let ads = if ea < a {
        atk_level + (1.0 - atk_level) * (ea / a)
    } else if ea < a + d {
        1.0 - (1.0 - sus) * (ea - a) / d
    } else {
        sus
    };
    let rel_start = if falling { ctx.now } else { ctx.state[2] };
    let rel_level = if falling { ads } else { ctx.state[3] };
    let y = if gate > 0.0 {
        ads
    } else {
        let er = crate::ir::now_wrap(ctx.now - rel_start);
        rel_level * (1.0 - er / r).max(0.0)
    };
    ctx.state[0] = gate;
    ctx.state[1] = atk_start;
    ctx.state[2] = rel_start;
    ctx.state[3] = rel_level;
    ctx.state[4] = atk_level;
    out[0] = clamp01(y);
}

fn tick_latch(ctx: &mut TickCtx, out: &mut [f32]) {
    // Triggered sample & hold: a rising edge (≤0 → >0, the shared trigger convention) captures
    // the input; the held value rests at 0 until the first trigger.
    let x = ctx.inputs[0];
    let trig = ctx.inputs[1];
    let edge = ctx.state[1] <= 0.0 && trig > 0.0;
    let held = if edge { x } else { ctx.state[0] };
    ctx.state[0] = held;
    ctx.state[1] = trig;
    out[0] = held;
}

fn tick_decay(ctx: &mut TickCtx, out: &mut [f32]) {
    // SC Decay: a leaky integrator whose pole gives a 60 dB decay over `time` seconds, exactly
    // `ringz`'s pole-radius idiom. r = 0.001^{1/(time·sr)} (time 0 ⇒ exponent +∞ ⇒ r = 0, a
    // NaN-free passthrough of the impulse); y = in + r·y₁ rings the input down and sums hits.
    // Pole cached in [key, r] behind y₁ (the filters' caching convention).
    let t = ctx.inputs[1].max(0.0);
    if ctx.state[1] != t + 1.0 {
        ctx.state[1] = t + 1.0;
        ctx.state[2] = powf(0.001, 1.0 / (t * ctx.sr));
    }
    let y = flush(ctx.inputs[0] + ctx.state[2] * ctx.state[0]);
    ctx.state[0] = y;
    out[0] = y;
}

fn tick_linseg(ctx: &mut TickCtx, out: &mut [f32]) {
    // Edge-triggered, clock-relative breakpoint envelope — the general form of `line`/`ar`.
    // The level/time list is inputs[1..]: a start level l0, then (time, level) pairs. A rising
    // edge captures `start = now`; the output is a pure function of e = now_wrap(now - start),
    // resting at 0 (`armed` gate) until first triggered. It plays once through the segments,
    // then holds the last level.
    let trig = ctx.inputs[0];
    let edge = ctx.state[1] <= 0.0 && trig > 0.0;
    let start = if edge { ctx.now } else { ctx.state[0] };
    let armed = if edge { 1.0 } else { ctx.state[2] };
    let e = crate::ir::now_wrap(ctx.now - start);
    let n = (ctx.inputs.len() - 2) / 2; // number of segments
    let mut acc = 0.0; // current segment's start time, in samples
    let mut prev = ctx.inputs[1]; // l0
    let mut level = ctx.inputs[ctx.inputs.len() - 1]; // default: hold the last level
    for i in 0..n {
        // At least one sample per segment (like `line`'s denom): a non-positive time is an
        // instant jump, never a divide by zero.
        let dur = (ctx.inputs[2 + 2 * i].max(0.0) * ctx.sr).max(1.0);
        let target = ctx.inputs[3 + 2 * i];
        let end = acc + dur;
        if e < end {
            level = prev + (e - acc) / dur * (target - prev); // lerp prev→target
            break; // segments past the active one can't matter — stop scanning
        }
        acc = end;
        prev = target;
    }
    ctx.state[0] = start;
    ctx.state[1] = trig;
    ctx.state[2] = armed;
    out[0] = armed * level;
}
