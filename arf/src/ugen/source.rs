//! Noise sources: white (`noise`), colored (`pink`, `brown`), shaped (`clipnoise`), random
//! impulses (`dust`, `dust2`), low-frequency modulation noise (`noiseh`, `noisei`), random
//! values (`rand`, `exprand`, `logrand` per note; `trand`, `texprand` per trigger), and the
//! stochastic trigger words (`coin`, `tchoose`). Every one draws its randomness from the shared
//! [`noise_sample`] counter hash, so it is deterministic per seed with no RNG state of its
//! own — exactly like `noise`.

use super::{Arity, Category, InputDescriptor, ListShape, TickCtx, UGen, Unit, signal, wrap01};
use crate::fastmath::powf;

pub(super) static UGENS: &[UGen] = &[
    // noise ( -- sig )   state: [sample counter]   full-scale white noise in [-1, 1)
    UGen {
        name: "noise",
        category: Category::Noise,
        description: "White-noise source — full-scale, in [-1, 1).",
        examples: &[
            "noise 0.2 * out",
            "noise 2000 lpf 0.3 * out",
            "noise  0.5 sine 1500 * 2000 +  lpf 0.3 * out",
        ],
        arity: Arity::Fixed(0),
        inputs: &[],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 6,
        tick: tick_noise,
    },
    // pink ( -- sig )   state: [b0..b6, counter]   Paul Kellet "refined" pink filter on white
    UGen {
        name: "pink",
        category: Category::Noise,
        description: "Pink noise — equal power per octave (−3 dB/oct).",
        examples: &["pink 0.4 * out", "pink 0.5 sine 0.5 * 0.5 + * 0.4 * out"],
        arity: Arity::Fixed(0),
        inputs: &[],
        outputs: 1,
        state_slots: 8,
        buffer_len: 0,
        cost: 18,
        tick: tick_pink,
    },
    // brown ( -- sig )   state: [z, counter]   reflected random walk, bounded in [-1, 1]
    UGen {
        name: "brown",
        category: Category::Noise,
        description: "Brown noise — a bounded random walk (−6 dB/oct).",
        examples: &["brown 0.4 * out", "brown 1200 lpf 0.4 * out"],
        arity: Arity::Fixed(0),
        inputs: &[],
        outputs: 1,
        state_slots: 2,
        buffer_len: 0,
        cost: 10,
        tick: tick_brown,
    },
    // clipnoise ( -- sig )   state: [counter]   two-level white: the sign of the white sample
    UGen {
        name: "clipnoise",
        category: Category::Noise,
        description: "Clipped white noise — randomly +1 or −1.",
        examples: &["clipnoise 0.15 * out", "clipnoise 1500 lpf 0.2 * out"],
        arity: Arity::Fixed(0),
        inputs: &[],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 7,
        tick: tick_clipnoise,
    },
    // dust ( density -- sig )   state: [counter]   unipolar random impulses
    UGen {
        name: "dust",
        category: Category::Noise,
        description: "Random impulses — unipolar [0, 1), `density` events per second.",
        examples: &[
            "20 dust 0.3 * out",
            "200 dust 0.2 * out",
            "12 dust 0.05 perc 440 sine * 0.3 * out",
        ],
        arity: Arity::Fixed(1),
        inputs: &[InputDescriptor {
            name: "density",
            unit: Unit::Hz,
            range: (0.0, 5_000.0),
            default: 100.0,
        }],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 8,
        tick: tick_dust,
    },
    // dust2 ( density -- sig )   state: [counter]   bipolar random impulses
    UGen {
        name: "dust2",
        category: Category::Noise,
        description: "Random impulses — bipolar [−1, 1), `density` events per second.",
        examples: &["30 dust2 0.3 * out", "300 dust2 0.2 * out"],
        arity: Arity::Fixed(1),
        inputs: &[InputDescriptor {
            name: "density",
            unit: Unit::Hz,
            range: (0.0, 5_000.0),
            default: 100.0,
        }],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 8,
        tick: tick_dust2,
    },
    // noiseh ( freq -- sig )   state: [phase, value, counter]   stepped sample-and-hold noise
    UGen {
        name: "noiseh",
        category: Category::Noise,
        description: "Stepped noise — holds a random value, jumping at `freq` Hz (sample-and-hold).",
        examples: &[
            "8 noiseh 0.3 * out",
            "10 noiseh 400 * 600 + sine 0.2 * out",
            "8000 noiseh 0.2 * out",
        ],
        arity: Arity::Fixed(1),
        inputs: &[InputDescriptor {
            name: "freq",
            unit: Unit::Hz,
            range: (0.0, 20_000.0),
            default: 8.0,
        }],
        outputs: 1,
        state_slots: 3,
        buffer_len: 0,
        cost: 8,
        tick: tick_noiseh,
    },
    // noisei ( freq -- sig )   state: [phase, prev, next, counter]   linearly-interpolated noise
    UGen {
        name: "noisei",
        category: Category::Noise,
        description: "Ramped noise — linearly interpolates between random values at `freq` Hz.",
        examples: &["6 noisei 0.3 * out", "5 noisei 300 * 500 + sine 0.2 * out"],
        arity: Arity::Fixed(1),
        inputs: &[InputDescriptor {
            name: "freq",
            unit: Unit::Hz,
            range: (0.0, 20_000.0),
            default: 8.0,
        }],
        outputs: 1,
        state_slots: 4,
        buffer_len: 0,
        cost: 10,
        tick: tick_noisei,
    },
    // rand ( lo hi -- sig )   state: [counter]   uniform draw, constant per instance: the counter
    // is read but never advanced, so the value holds for the note's whole life.
    UGen {
        name: "rand",
        category: Category::Noise,
        description: "Random constant — a uniform draw in [lo, hi), held for the life of the note.",
        examples: &[
            "200 800 rand sine 0.2 * out",
            "220 saw 400 2000 rand lpf 0.3 * out",
        ],
        arity: Arity::Fixed(2),
        inputs: &[signal("lo"), signal("hi")],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 6,
        tick: tick_rand,
    },
    // exprand ( lo hi -- sig )   state: [counter]   per-note draw, exponential bias toward lo
    UGen {
        name: "exprand",
        category: Category::Noise,
        description: "Random constant, biased low — lo·(hi/lo)^u, held for the note; args must be positive.",
        examples: &[
            "200 4000 exprand sine 0.2 * out",
            "110 saw 300 6000 exprand lpf 0.3 * out",
        ],
        arity: Arity::Fixed(2),
        inputs: &[signal("lo"), signal("hi")],
        outputs: 1,
        state_slots: 4,
        buffer_len: 0,
        cost: 14,
        tick: tick_exprand,
    },
    // logrand ( lo hi -- sig )   state: [counter]   per-note draw, exponential bias toward hi
    UGen {
        name: "logrand",
        category: Category::Noise,
        description: "Random constant, biased high — hi·(lo/hi)^u, held for the note; args must be positive.",
        examples: &["200 4000 logrand sine 0.2 * out"],
        arity: Arity::Fixed(2),
        inputs: &[signal("lo"), signal("hi")],
        outputs: 1,
        state_slots: 4,
        buffer_len: 0,
        cost: 14,
        tick: tick_logrand,
    },
    // trand ( trig lo hi -- sig )   state: [held, prev, armed, counter]   uniform redraw on each
    // rising edge; draws once at the first sample so it never rests outside [lo, hi)
    UGen {
        name: "trand",
        category: Category::Noise,
        description: "Triggered random — holds a uniform draw in [lo, hi), redrawn on each rising trigger edge (draws at note start).",
        examples: &[
            "8 impulse 200 800 trand sine 0.2 * out",
            "2 sine trig 300 600 trand sine 0.2 * out",
        ],
        arity: Arity::Fixed(3),
        inputs: &[signal("trig"), signal("lo"), signal("hi")],
        outputs: 1,
        state_slots: 4,
        buffer_len: 0,
        cost: 8,
        tick: tick_trand,
    },
    // texprand ( trig lo hi -- sig )   state: [held, prev, armed, counter]   exponential redraw
    UGen {
        name: "texprand",
        category: Category::Noise,
        description: "Triggered random, biased low — lo·(hi/lo)^u per rising trigger edge; args must be positive.",
        examples: &["8 impulse 200 3200 texprand sine 0.2 * out"],
        arity: Arity::Fixed(3),
        inputs: &[signal("trig"), signal("lo"), signal("hi")],
        outputs: 1,
        state_slots: 4,
        buffer_len: 0,
        cost: 16,
        tick: tick_texprand,
    },
    // coin ( trig prob -- trig )   state: [prev, counter]   probability gate for triggers
    UGen {
        name: "coin",
        category: Category::Trigger,
        description: "Probability gate — passes each rising trigger edge with probability `prob`, swallows it otherwise.",
        examples: &[
            "8 clock 0.6 coin 0.04 perc noise * 0.3 * out",
            "8 impulse 0.5 coin 0.05 perc 440 sine * 0.3 * out",
        ],
        arity: Arity::Fixed(2),
        inputs: &[
            signal("trig"),
            InputDescriptor {
                name: "prob",
                unit: Unit::Ratio,
                range: (0.0, 1.0),
                default: 0.5,
            },
        ],
        outputs: 1,
        state_slots: 2,
        buffer_len: 0,
        cost: 6,
        tick: tick_coin,
    },
    // tchoose ( trig [v0 v1 …] -- val )   state: [index, prev, armed, counter]   the random `seq`:
    // built by a front-end's `VariadicLed` arm — inputs[0] is the trigger, inputs[1..] the values.
    UGen {
        name: "tchoose",
        category: Category::Trigger,
        description: "Random choice — holds a uniformly picked value from the list, repicking on each trigger (picks at note start).",
        examples: &[
            "4 impulse [ 220 330 440 660 ] tchoose sine 0.2 * out",
            "8 impulse [ 60 63 67 70 ] tchoose mtof saw 800 lpf 0.2 * out",
        ],
        arity: Arity::VariadicLed {
            shape: ListShape::Any,
        },
        inputs: &[signal("trig")],
        outputs: 1,
        state_slots: 4,
        buffer_len: 0,
        cost: 6,
        tick: tick_tchoose,
    },
];

/// Wellons' `lowbias32` integer hash — the avalanche shared by [`noise_sample`] (the white-noise
/// mapping) and [`noise_seed`] (the per-instance seed scatter), so both derive from one mixer.
#[inline]
fn lowbias32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

/// Map a sample counter to a white-noise sample in [-1, 1). A pure integer hash
/// (Wellons' `lowbias32`), so it has no state of its own — the single source of truth
/// for every noise source's randomness. Only the top 24 bits feed the float, so the
/// `u32 → f32` is exact (no rounding) and the result is strictly below 1.0.
pub(crate) fn noise_sample(counter: u32) -> f32 {
    ((lowbias32(counter) >> 8) as f32) * (2.0 / 16_777_216.0) - 1.0
}

/// A per-instance seed for a noise source's sample counter: scatter `ordinal` (the compiler's
/// running index of seeded ops) across the counter space via the same avalanche, returned as the
/// f32 integer to drop into the counter slot (`< 2^24`, so the slot's round-trip is lossless).
/// Distinct ordinals land far apart, so co-existing instances read different regions of the one
/// shared sequence — without this every `noise`/`dust`/… starts at counter 0 and emits the
/// identical stream. The compiler assigns it as the slot's fresh init.
pub(crate) fn noise_seed(ordinal: u32) -> f32 {
    (lowbias32(ordinal) & COUNTER_MASK) as f32
}

/// The state slot holding the per-instance sample counter, for the noise sources that have one
/// (`None` for every other UGen). The compiler seeds this slot via [`noise_seed`] so co-existing
/// instances decorrelate. Co-located with the rows above so a new noise source is registered here
/// in the same file; each index matches that row's `state:` layout comment.
pub(crate) fn seed_slot(name: &str) -> Option<u8> {
    match name {
        "noise" | "clipnoise" | "dust" | "dust2" | "rand" | "exprand" | "logrand" => Some(0),
        "brown" | "coin" => Some(1),
        "noiseh" => Some(2),
        "noisei" | "trand" | "texprand" | "tchoose" => Some(3),
        "pink" => Some(7),
        _ => None,
    }
}

fn tick_noise(ctx: &mut TickCtx, out: &mut [f32]) {
    // The counter lives in the f32 state slot as an exact integer < 2^24, so the casts
    // round-trip losslessly. It just keeps counting across a re-eval (like a phase).
    let counter = ctx.state[0] as u32;
    out[0] = noise_sample(counter);
    ctx.state[0] = (counter.wrapping_add(1) & 0x00FF_FFFF) as f32;
}

/// The 24-bit wrap every noise source applies to its sample counter: kept below 2^24 so the
/// counter's f32 round-trip in a state slot is lossless (matching `noise`).
const COUNTER_MASK: u32 = 0x00FF_FFFF;

/// Advance a noise counter one step, wrapped to 24 bits so the f32 slot round-trip stays
/// lossless.
fn advance_counter(c: u32) -> u32 {
    c.wrapping_add(1) & COUNTER_MASK
}

// Paul Kellet's "refined" pink-noise filter: six leaky one-pole sections plus a feed-forward
// term, summed and scaled. Coefficients live here as `tick_pink`'s single source. The
// trailing `0.11` scale brings the output to a comfortable level (the canonical Kellet
// constant) for white in [-1, 1).
const PINK_COEFFS: [f32; 6] = [0.99886, 0.99332, 0.96900, 0.86650, 0.55000, -0.7616];
#[allow(clippy::excessive_precision)] // Kellet's published constants, kept verbatim
const PINK_GAINS: [f32; 6] = [
    0.0555179, 0.0750759, 0.1538520, 0.3104856, 0.5329522, -0.0168980,
];
const PINK_B6_GAIN: f32 = 0.115926;
const PINK_WHITE_GAIN: f32 = 0.5362;
const PINK_SCALE: f32 = 0.11;

fn tick_pink(ctx: &mut TickCtx, out: &mut [f32]) {
    // Run the six poles (each `coeff·prev + white·gain`; the sixth folds its minus sign into a
    // negative gain so every section shares one shape), then sum them with the held b6 and a
    // direct white term in a fixed left-to-right order for f32 determinism.
    let counter = ctx.state[7] as u32;
    let w = noise_sample(counter);
    let mut poles = [0.0f32; 6];
    for (i, pole) in poles.iter_mut().enumerate() {
        *pole = PINK_COEFFS[i] * ctx.state[i] + w * PINK_GAINS[i];
    }
    let mut pink = poles[0];
    for &p in &poles[1..] {
        pink += p;
    }
    pink += ctx.state[6]; // held b6 from the previous sample
    pink += w * PINK_WHITE_GAIN;
    for (slot, &pole) in poles.iter().enumerate() {
        ctx.state[slot] = pole;
    }
    ctx.state[6] = w * PINK_B6_GAIN;
    out[0] = pink * PINK_SCALE;
    ctx.state[7] = advance_counter(counter) as f32;
}

fn tick_brown(ctx: &mut TickCtx, out: &mut [f32]) {
    // A random walk stepped by white/8 and reflected at the rails, so it stays bounded in
    // [-1, 1] without the precision drift of an unbounded accumulator. The step magnitude is
    // < 2, so one reflection per rail always lands the value back in range.
    let counter = ctx.state[1] as u32;
    let w = noise_sample(counter);
    let stepped = ctx.state[0] + w * 0.125;
    let folded_hi = if stepped > 1.0 {
        2.0 - stepped
    } else {
        stepped
    };
    let z = if folded_hi < -1.0 {
        -2.0 - folded_hi
    } else {
        folded_hi
    };
    ctx.state[0] = z;
    out[0] = z;
    ctx.state[1] = advance_counter(counter) as f32;
}

fn tick_clipnoise(ctx: &mut TickCtx, out: &mut [f32]) {
    let counter = ctx.state[0] as u32;
    let w = noise_sample(counter);
    out[0] = if w >= 0.0 { 1.0 } else { -1.0 };
    ctx.state[0] = advance_counter(counter) as f32;
}

fn tick_dust(ctx: &mut TickCtx, out: &mut [f32]) {
    // Fire with probability density/sr each sample; the impulse height is the uniform draw
    // rescaled by 1/threshold, so heights spread over [0, 1) — the classic `Dust` shape.
    let density = ctx.inputs[0].max(0.0);
    let counter = ctx.state[0] as u32;
    let w = noise_sample(counter);
    let u = w * 0.5 + 0.5;
    let thresh = density / ctx.sr;
    out[0] = if u < thresh { u / thresh } else { 0.0 };
    ctx.state[0] = advance_counter(counter) as f32;
}

fn tick_dust2(ctx: &mut TickCtx, out: &mut [f32]) {
    let density = ctx.inputs[0].max(0.0);
    let counter = ctx.state[0] as u32;
    let w = noise_sample(counter);
    let u = w * 0.5 + 0.5;
    let thresh = density / ctx.sr;
    out[0] = if u < thresh {
        2.0 * (u / thresh) - 1.0
    } else {
        0.0
    };
    ctx.state[0] = advance_counter(counter) as f32;
}

fn tick_noiseh(ctx: &mut TickCtx, out: &mut [f32]) {
    // Sample-and-hold: advance a phase by freq/sr; whenever it crosses 1, latch a fresh random
    // draw and advance the counter, otherwise hold both. Starts at 0 until the first wrap.
    let counter = ctx.state[2] as u32;
    let next_counter = advance_counter(counter);
    let draw = noise_sample(next_counter);
    let phase = ctx.state[0] + ctx.inputs[0] / ctx.sr;
    let wrapped = phase >= 1.0;
    let value = if wrapped { draw } else { ctx.state[1] };
    ctx.state[0] = wrap01(phase);
    ctx.state[1] = value;
    ctx.state[2] = (if wrapped { next_counter } else { counter }) as f32;
    out[0] = value;
}

fn tick_noisei(ctx: &mut TickCtx, out: &mut [f32]) {
    // Like `noiseh`, but linearly interpolate from the previously latched value to the next as
    // the phase ramps 0→1: on a wrap the old `next` becomes `prev` and a fresh draw becomes
    // `next`. Starts at 0 and ramps from there until the first wrap.
    let counter = ctx.state[3] as u32;
    let next_counter = advance_counter(counter);
    let draw = noise_sample(next_counter);
    let phase = ctx.state[0] + ctx.inputs[0] / ctx.sr;
    let wrapped = phase >= 1.0;
    let prev = if wrapped { ctx.state[2] } else { ctx.state[1] };
    let next = if wrapped { draw } else { ctx.state[2] };
    let frac = wrap01(phase);
    ctx.state[0] = frac;
    ctx.state[1] = prev;
    ctx.state[2] = next;
    ctx.state[3] = (if wrapped { next_counter } else { counter }) as f32;
    out[0] = prev + frac * (next - prev);
}

/// The unit draw every random-value source maps from: the counter's white sample folded
/// into [0, 1). One definition so `rand`/`trand` and their exp/log variants agree on it.
/// The counter is salted first because 0 is a fixed point of `lowbias32` — the first seeded
/// instance in a program (ordinal 0, seed 0) would otherwise always draw exactly 0, and
/// unlike the streaming noises a held draw makes that degenerate value audible.
fn unit_draw(counter: u32) -> f32 {
    noise_sample(counter ^ 0x9e37_79b9) * 0.5 + 0.5
}

/// Clamp a `rand`-family bound to be strictly positive, so the exp/log mappings' `powf`
/// ratio can never go negative or divide by zero (mirrors `perc`'s non-negative guard).
fn positive(x: f32) -> f32 {
    x.max(1e-6)
}

fn tick_rand(ctx: &mut TickCtx, out: &mut [f32]) {
    // The per-instance seed IS the value: read the counter, never advance it, so the draw
    // holds for the note's whole life.
    let u = unit_draw(ctx.state[0] as u32);
    out[0] = ctx.inputs[0] + (ctx.inputs[1] - ctx.inputs[0]) * u;
}

// The draw is fixed for the instance's life (the counter never advances), so exprand/
// logrand's mapped value only changes with the bounds: cache [key_lo, key_hi, value]
// behind the counter slot. Both keys are clamped ≥ 1e-6 by `positive`, so the zero-filled
// fresh state always misses and the first tick computes (the filters' caching convention).
fn tick_exprand(ctx: &mut TickCtx, out: &mut [f32]) {
    let lo = positive(ctx.inputs[0]);
    let hi = positive(ctx.inputs[1]);
    if ctx.state[1] != lo || ctx.state[2] != hi {
        let u = unit_draw(ctx.state[0] as u32);
        ctx.state[1] = lo;
        ctx.state[2] = hi;
        ctx.state[3] = lo * powf(hi / lo, u);
    }
    out[0] = ctx.state[3];
}

fn tick_logrand(ctx: &mut TickCtx, out: &mut [f32]) {
    let lo = positive(ctx.inputs[0]);
    let hi = positive(ctx.inputs[1]);
    if ctx.state[1] != lo || ctx.state[2] != hi {
        let u = unit_draw(ctx.state[0] as u32);
        ctx.state[1] = lo;
        ctx.state[2] = hi;
        ctx.state[3] = hi * powf(lo / hi, u);
    }
    out[0] = ctx.state[3];
}

/// The shared triggered-random core: redraw a unit sample on each rising edge (and once at
/// the first sample, so the output never rests outside the range), map it through `map`,
/// and hold. State: [held, prev, armed, counter].
fn trand_core(ctx: &mut TickCtx, map: fn(f32, f32, f32) -> f32) -> f32 {
    let trig = ctx.inputs[0];
    let edge = ctx.state[1] <= 0.0 && trig > 0.0;
    let draw = edge || ctx.state[2] == 0.0;
    let counter = ctx.state[3] as u32;
    let held = if draw {
        map(ctx.inputs[1], ctx.inputs[2], unit_draw(counter))
    } else {
        ctx.state[0]
    };
    ctx.state[0] = held;
    ctx.state[1] = trig;
    ctx.state[2] = 1.0;
    ctx.state[3] = (if draw {
        advance_counter(counter)
    } else {
        counter
    }) as f32;
    held
}

fn tick_trand(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = trand_core(ctx, |lo, hi, u| lo + (hi - lo) * u);
}

fn tick_texprand(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = trand_core(ctx, |lo, hi, u| {
        positive(lo) * powf(positive(hi) / positive(lo), u)
    });
}

fn tick_coin(ctx: &mut TickCtx, out: &mut [f32]) {
    // A rising edge flips the coin; a win passes the trigger sample through (its height
    // intact, so a `dust` impulse keeps its amplitude), a loss swallows it. The counter
    // only advances on flips, so the stream is consumed per event like `noiseh`'s.
    let trig = ctx.inputs[0];
    let edge = ctx.state[0] <= 0.0 && trig > 0.0;
    let counter = ctx.state[1] as u32;
    out[0] = if edge && unit_draw(counter) < ctx.inputs[1] {
        trig
    } else {
        0.0
    };
    ctx.state[0] = trig;
    ctx.state[1] = (if edge {
        advance_counter(counter)
    } else {
        counter
    }) as f32;
}

fn tick_tchoose(ctx: &mut TickCtx, out: &mut [f32]) {
    // The random `seq`: a rising edge (and the first sample, so the hold is always a real
    // list value) picks a uniform index into the value-list and holds it.
    let n = ctx.inputs.len() - 1; // number of values (inputs[0] is the trigger)
    let trig = ctx.inputs[0];
    let edge = ctx.state[1] <= 0.0 && trig > 0.0;
    let draw = edge || ctx.state[2] == 0.0;
    let counter = ctx.state[3] as u32;
    let index = if draw {
        ((unit_draw(counter) * n as f32) as usize).min(n - 1) as f32
    } else {
        ctx.state[0]
    };
    ctx.state[0] = index;
    ctx.state[1] = trig;
    ctx.state[2] = 1.0;
    ctx.state[3] = (if draw {
        advance_counter(counter)
    } else {
        counter
    }) as f32;
    out[0] = ctx.inputs[1 + index as usize];
}
