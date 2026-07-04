//! Arithmetic and scalar math: `+ - * /`, `mix`, `select`, the scalar transforms (`min`/`max`/`clip`/
//! `abs`/`neg`/`sign`/`recip`/`sqrt`/`floor`/`ceil`/`round`/`trunc`/`exp`), the trig words
//! (`sin`/`cos`/`tan`/`atan`), the range helpers (`linlin`/`range`/`lerp`/`xfade`/`uni`/`bi`),
//! the unit conversions (`mtof`/`ftom`/`dbamp`/`ampdb`), and the comparators (`<` `>` `<=`
//! `>=` `==` `!=`). Stateless; each `tick` is one IEEE expression
//! and each `emit` is its bit-exact CLIF mirror (native ops or the shared shims — never raw
//! `fmin`/`fmax`, which would propagate NaN where the VM suppresses it).

use super::{signal, Arity, Category, ListShape, Rate, TickCtx, UGen};

pub(super) static UGENS: &[UGen] = &[
    // + ( a b -- a+b )
    UGen { name: "+", category: Category::Math, description: "Add two signals.",
           examples: &["440 sine 660 sine + 0.2 * out", "1 2 + out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_add },
    // - ( a b -- a-b )
    UGen { name: "-", category: Category::Math, description: "Subtract the second signal from the first.",
           examples: &["noise 0.2 *  noise 0.2 * - out", "10 3 - out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_sub },
    // * ( a b -- a*b )
    UGen { name: "*", category: Category::Math, description: "Multiply two signals (gain, ring/amplitude modulation).",
           examples: &["440 sine 0.2 * out", "440 sine  110 sine *  0.3 * out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_mul },
    // / ( a b -- a/b )   IEEE: a/0 is ±inf, 0/0 is NaN — identical on both backends
    UGen { name: "/", category: Category::Math, description: "Divide the first signal by the second (IEEE: ÷0 is ±inf).",
           examples: &["440 sine  4 /  0.2 * out", "880 2 / sine 0.2 * out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 4, tick: tick_div },
    // mix ( chans -- sig )   sum a whole channel-list to mono; variadic (consumes the list)
    UGen { name: "mix", category: Category::Math, description: "Sum a channel-list down to one signal.",
           examples: &["[ 220 330 440 ] sine mix 0.1 * out", "[ 110 220 ] saw mix 600 lpf 0.2 * out"], arity: Arity::Variadic, inputs: &[signal("in")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 2, tick: tick_mix },
    // select ( index chans -- sig )   pick one channel by index; variadic, built by a bespoke
    // front-end `VariadicLed` arm: input 0 is the index, inputs 1.. the values.
    UGen { name: "select", category: Category::Math, description: "Pick one signal from a channel-list by index (floored, clamped to the list) — `index [ a b c ] select`.",
           examples: &["1.7 [ 10 20 30 ] select out", "1 phasor 3 * [ 110 220 330 ] select sine 0.2 * out"], arity: Arity::VariadicLed { shape: ListShape::Any }, inputs: &[signal("index")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 3, tick: tick_select },

    // --- scalar transforms ---------------------------------------------------
    // min ( a b -- min )   NaN-suppressing (via the shared shim), like the VM's `f32::min`.
    UGen { name: "min", category: Category::Math, description: "The smaller of two signals.",
           examples: &["440 sine  0.3 min  out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_min },
    // max ( a b -- max )
    UGen { name: "max", category: Category::Math, description: "The larger of two signals.",
           examples: &["440 sine  -0.3 max  0.5 * out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_max },
    // clip ( x lo hi -- y )   clamp = max(lo, x) then min(hi, .); the workhorse limiter.
    UGen { name: "clip", category: Category::Math, description: "Clamp a signal into [lo, hi].",
           examples: &["440 sine 3 *  -0.4 0.4 clip  0.5 * out"], arity: Arity::Fixed(3), inputs: &[signal("x"), signal("lo"), signal("hi")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 2, tick: tick_clip },
    // abs ( x -- |x| )
    UGen { name: "abs", category: Category::Math, description: "Absolute value (magnitude).",
           examples: &["440 sine abs 0.3 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_abs },
    // neg ( x -- -x )
    UGen { name: "neg", category: Category::Math, description: "Negate a signal (flip its sign).",
           examples: &["440 sine neg 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_neg },
    // sign ( x -- s )   -1, 0, or +1.
    UGen { name: "sign", category: Category::Math, description: "Sign of a signal: -1, 0, or +1.",
           examples: &["440 sine sign 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_sign },
    // recip ( x -- 1/x )   IEEE: 1/0 is ±inf.
    UGen { name: "recip", category: Category::Math, description: "Reciprocal 1/x (IEEE: 1/0 is ±inf).",
           examples: &["0.5 recip 220 * sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 4, tick: tick_recip },
    // sqrt ( x -- √x )   IEEE: √(negative) is NaN.
    UGen { name: "sqrt", category: Category::Math, description: "Square root (IEEE: √negative is NaN).",
           examples: &["2 sqrt 220 * sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 4, tick: tick_sqrt },
    // floor ( x -- ⌊x⌋ )
    UGen { name: "floor", category: Category::Math, description: "Round toward −∞.",
           examples: &["1 phasor 5 * floor 110 * sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_floor },
    // ceil ( x -- ⌈x⌉ )
    UGen { name: "ceil", category: Category::Math, description: "Round toward +∞.",
           examples: &["1 phasor 4 * ceil 110 * sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_ceil },
    // round ( x -- y )   nearest integer, ties to even (matches the JIT's CLIF `nearest`).
    UGen { name: "round", category: Category::Math, description: "Round to nearest integer (ties to even).",
           examples: &["440 sine 4 * round 4 / 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_round },
    // trunc ( x -- y )   round toward zero.
    UGen { name: "trunc", category: Category::Math, description: "Round toward zero (drop the fraction).",
           examples: &["1 phasor 8 * trunc 55 * sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_trunc },
    // exp ( x -- e^x )   via the shared exp shim (bit-exact with the VM).
    UGen { name: "exp", category: Category::Math, description: "Exponential e^x.",
           examples: &["4 impulse 0.2 perc neg exp 0.4 *  440 sine *  0.3 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_exp },

    // --- trig (shim-backed; radians) ------------------------------------------
    // sin ( x -- sin x )   the plain function (waveshaping/panning laws), not the oscillator.
    UGen { name: "sin", category: Category::Math, description: "Sine of x (radians).",
           examples: &["440 sine 3 * sin 0.3 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_sin },
    // cos ( x -- cos x )
    UGen { name: "cos", category: Category::Math, description: "Cosine of x (radians).",
           examples: &["440 sine 3 * cos 0.3 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_cos },
    // tan ( x -- tan x )   IEEE: unbounded near odd multiples of π/2.
    UGen { name: "tan", category: Category::Math, description: "Tangent of x (radians).",
           examples: &["440 sine tan -1 1 clip 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_tan },
    // atan ( x -- atan x )   bounded ±π/2 — a gentle soft-clip curve.
    UGen { name: "atan", category: Category::Math, description: "Arctangent of x (radians, bounded ±π/2).",
           examples: &["110 saw 5 * atan 0.3 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_atan },

    // --- range & interpolation -----------------------------------------------
    // linlin ( x ilo ihi olo ohi -- y )   map x linearly from [ilo,ihi] onto [olo,ohi].
    UGen { name: "linlin", category: Category::Math, description: "Linearly map x from [ilo,ihi] to [olo,ohi].",
           examples: &["3 sine  -1 1 200 800 linlin  220 saw swap lpf  0.3 * out", "0.5 0 1 0 10 linlin out"], arity: Arity::Fixed(5),
           inputs: &[signal("x"), signal("ilo"), signal("ihi"), signal("olo"), signal("ohi")],
           outputs: 1, state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 5, tick: tick_linlin },
    // range ( x lo hi -- y )   map a bipolar -1..1 signal (e.g. an LFO) onto [lo,hi].
    UGen { name: "range", category: Category::Math, description: "Map a bipolar -1..1 signal onto [lo,hi].",
           examples: &["110 saw  1 sine 0 1800 range  0.8 bpf  0.3 * out", "0.2 sine 200 500 range sine 0.2 * out"], arity: Arity::Fixed(3), inputs: &[signal("x"), signal("lo"), signal("hi")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 4, tick: tick_range },
    // lerp ( a b t -- y )   linear crossfade a→b by t.
    UGen { name: "lerp", category: Category::Math, description: "Linear interpolate a→b by t.",
           examples: &["220 440  0.5 sine uni  lerp sine 0.2 * out", "10 20 0.5 lerp out"], arity: Arity::Fixed(3), inputs: &[signal("a"), signal("b"), signal("t")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 3, tick: tick_lerp },
    // xfade ( a b t -- y )   equal-power crossfade a→b by t in 0..1 (each leg −3 dB at 0.5).
    UGen { name: "xfade", category: Category::Math, description: "Equal-power crossfade a→b by t (0..1).",
           examples: &["220 sine  noise  0.1 sine uni  xfade  0.3 * out"], arity: Arity::Fixed(3), inputs: &[signal("a"), signal("b"), signal("t")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_xfade },
    // uni ( x -- y )   bipolar -1..1 → unipolar 0..1.
    UGen { name: "uni", category: Category::Math, description: "Bipolar -1..1 to unipolar 0..1.",
           examples: &["440 sine  5 sine uni *  0.2 * out", "0.15 tri uni 400 * 500 + sine 0.3 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 2, tick: tick_uni },
    // bi ( x -- y )   unipolar 0..1 → bipolar -1..1.
    UGen { name: "bi", category: Category::Math, description: "Unipolar 0..1 to bipolar -1..1.",
           examples: &["1 phasor bi 200 * 400 + sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 2, tick: tick_bi },

    // --- transcendentals & wrapping (shim-backed) ----------------------------
    // pow ( x y -- x^y )
    UGen { name: "pow", category: Category::Math, description: "Raise x to the power y.",
           examples: &["440 sine uni 2 pow bi 0.3 * out"], arity: Arity::Fixed(2), inputs: &[signal("x"), signal("y")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_pow },
    // log ( x -- ln x )   natural logarithm (IEEE: log of ≤0 is NaN/-inf).
    UGen { name: "log", category: Category::Math, description: "Natural logarithm ln(x).",
           examples: &["2.718281828 log 440 * sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_log },
    // log2 ( x -- log2 x )
    UGen { name: "log2", category: Category::Math, description: "Base-2 logarithm.",
           examples: &["880 440 / log2 440 * sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_log2 },
    // log10 ( x -- log10 x )
    UGen { name: "log10", category: Category::Math, description: "Base-10 logarithm.",
           examples: &["1000 log10 220 * sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("x")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_log10 },
    // mod ( a b -- y )   euclidean remainder: the result has b's sign convention and is never
    // negative for b>0 (unlike `%`), so it is safe for phase/counter wrapping.
    UGen { name: "mod", category: Category::Math, description: "Euclidean remainder a mod b (never negative for b>0).",
           examples: &["1 phasor 4 *  1 mod  bi 200 * 400 + sine 0.2 * out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 4, tick: tick_mod },
    // wrap ( x lo hi -- y )   wrap x into the half-open range [lo, hi).
    UGen { name: "wrap", category: Category::Math, description: "Wrap x into the range [lo, hi).",
           examples: &["220 sine 1.5 *  -0.4 0.4 wrap  0.5 * out"], arity: Arity::Fixed(3), inputs: &[signal("x"), signal("lo"), signal("hi")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 5, tick: tick_wrap },
    // fold ( x lo hi -- y )   reflect x back and forth within [lo, hi] (a wavefolder).
    UGen { name: "fold", category: Category::Math, description: "Reflect x into [lo, hi] (wavefolder).",
           examples: &["220 sine 3 *  -0.4 0.4 fold  0.5 * out"], arity: Arity::Fixed(3), inputs: &[signal("x"), signal("lo"), signal("hi")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 6, tick: tick_fold },
    // linexp ( x ilo ihi olo ohi -- y )   map x linearly from [ilo,ihi] onto an exponential
    // [olo,ohi] (olo,ohi > 0) — the natural curve for frequency/amplitude control.
    UGen { name: "linexp", category: Category::Math, description: "Map x from [ilo,ihi] to an exponential [olo,ohi].",
           examples: &["1 phasor 0 1 55 880 linexp sine 0.2 * out", "0.2 sine uni 0 1 200 2000 linexp 220 saw swap lpf 0.3 * out"], arity: Arity::Fixed(5),
           inputs: &[signal("x"), signal("ilo"), signal("ihi"), signal("olo"), signal("ohi")],
           outputs: 1, state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 12, tick: tick_linexp },

    // --- unit conversions (shim-backed) ----------------------------------------
    // mtof ( m -- hz )   MIDI note → Hz (A4 = 69 = 440 Hz). Via powf(2, ·) so the emit
    // mirrors it with the shared pow shim.
    UGen { name: "mtof", category: Category::Math, description: "MIDI note number to frequency in Hz (69 = 440).",
           examples: &["60 mtof sine 0.2 * out", "4 impulse [ 60 63 67 70 ] seq mtof saw 600 lpf 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("m")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_mtof },
    // ftom ( hz -- m )   Hz → MIDI note (IEEE: hz ≤ 0 is NaN/-inf, like log2).
    UGen { name: "ftom", category: Category::Math, description: "Frequency in Hz to MIDI note number (440 = 69).",
           examples: &["440 ftom round mtof sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("hz")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_ftom },
    // dbamp ( db -- amp )   decibels → linear gain (0 dB = 1).
    UGen { name: "dbamp", category: Category::Math, description: "Decibels to linear amplitude (0 dB = 1).",
           examples: &["440 sine  -12 dbamp *  out"], arity: Arity::Fixed(1), inputs: &[signal("db")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_dbamp },
    // ampdb ( amp -- db )   linear gain → decibels (IEEE: amp ≤ 0 is NaN/-inf, like log10).
    UGen { name: "ampdb", category: Category::Math, description: "Linear amplitude to decibels (1 = 0 dB).",
           examples: &["0.5 ampdb -60 / 220 * sine 0.2 * out"], arity: Arity::Fixed(1), inputs: &[signal("amp")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 10, tick: tick_ampdb },

    // --- comparators -----------------------------------------------------------
    // Each outputs 1.0 when the relation holds, 0.0 otherwise. Ordered IEEE compares
    // (NaN ⇒ 0.0) — except `!=`, which is unordered like Rust's `!=` (NaN ⇒ 1.0).
    // < ( a b -- a<b )
    UGen { name: "<", category: Category::Math, description: "1 if a < b, else 0.",
           examples: &["1 phasor 0.5 <  440 sine *  0.2 * out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_lt },
    // > ( a b -- a>b )
    UGen { name: ">", category: Category::Math, description: "1 if a > b, else 0.",
           examples: &["2 sine 0 >  110 saw *  0.3 * out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_gt },
    // <= ( a b -- a<=b )
    UGen { name: "<=", category: Category::Math, description: "1 if a <= b, else 0.",
           examples: &["1 phasor 0.25 <=  440 sine *  0.2 * out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_le },
    // >= ( a b -- a>=b )
    UGen { name: ">=", category: Category::Math, description: "1 if a >= b, else 0.",
           examples: &["1 phasor 0.5 >=  220 saw *  0.3 * out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_ge },
    // == ( a b -- a==b )
    UGen { name: "==", category: Category::Math, description: "1 if a equals b, else 0.",
           examples: &["1 phasor 8 * floor  0 ==  440 sine *  0.2 * out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_eq },
    // != ( a b -- a!=b )
    UGen { name: "!=", category: Category::Math, description: "1 if a differs from b, else 0.",
           examples: &["1 phasor 4 * floor  0 !=  220 saw *  0.3 * out"], arity: Arity::Fixed(2), inputs: &[signal("a"), signal("b")], outputs: 1,
           state_slots: 0, buffer_len: 0, rate: Rate::Audio, cost: 1, tick: tick_ne },
];

fn tick_add(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0] + ctx.inputs[1];
}

fn tick_sub(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0] - ctx.inputs[1];
}

fn tick_mul(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0] * ctx.inputs[1];
}

fn tick_div(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0] / ctx.inputs[1];
}

fn tick_mix(ctx: &mut TickCtx, out: &mut [f32]) {
    // Left-fold from the first channel (matching `emit_mix`), so `[a b] mix` is bit-identical
    // to `a b +` and a single channel passes straight through. Inputs are non-empty (a variadic
    // word consumes a non-empty channel-list).
    let mut acc = ctx.inputs[0];
    for &x in &ctx.inputs[1..] {
        acc += x;
    }
    out[0] = acc;
}

// Not `.clamp()`: the index bounds mirror the JIT's NaN-suppressing max/min shims.
#[allow(clippy::manual_clamp)]
fn tick_select(ctx: &mut TickCtx, out: &mut [f32]) {
    // Floored and clamped into the list (like SC's `Select`): inputs[0] is the index,
    // inputs[1..] the values. The `.max(0.0)` also de-NaNs the index, so a NaN picks
    // values[0] on both backends. The clamp keeps the index an integer-valued f32 in
    // 0..n, which is what makes `emit_select`'s equality chain exact.
    let n = (ctx.inputs.len() - 1) as f32;
    let i = ctx.inputs[0].floor().max(0.0).min(n - 1.0);
    out[0] = ctx.inputs[1 + i as usize];
}

fn tick_min(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].min(ctx.inputs[1]);
}

fn tick_max(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].max(ctx.inputs[1]);
}

fn tick_clip(ctx: &mut TickCtx, out: &mut [f32]) {
    // max(lo) then min(hi) — same order as `emit_clip`, both via NaN-suppressing min/max.
    out[0] = ctx.inputs[0].max(ctx.inputs[1]).min(ctx.inputs[2]);
}

fn tick_abs(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].abs();
}

fn tick_neg(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = -ctx.inputs[0];
}

fn tick_sign(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    // Ordered compares (NaN ⇒ neither branch ⇒ 0), matching `emit_sign`'s `select_gt`/`select_lt`.
    out[0] = if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    };
}

fn tick_recip(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = 1.0 / ctx.inputs[0];
}

fn tick_sqrt(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].sqrt();
}

fn tick_floor(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].floor();
}

fn tick_ceil(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].ceil();
}

fn tick_round(ctx: &mut TickCtx, out: &mut [f32]) {
    // Ties to even, matching the JIT's CLIF `nearest` — NOT `f32::round` (ties away from zero).
    out[0] = ctx.inputs[0].round_ties_even();
}

fn tick_trunc(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].trunc();
}

fn tick_exp(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].exp();
}

fn tick_sin(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].sin();
}

fn tick_cos(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].cos();
}

fn tick_tan(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].tan();
}

fn tick_atan(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].atan();
}

fn tick_linlin(ctx: &mut TickCtx, out: &mut [f32]) {
    let [x, ilo, ihi, olo, ohi] = [ctx.inputs[0], ctx.inputs[1], ctx.inputs[2], ctx.inputs[3], ctx.inputs[4]];
    out[0] = olo + (x - ilo) * (ohi - olo) / (ihi - ilo);
}

fn tick_range(ctx: &mut TickCtx, out: &mut [f32]) {
    let [x, lo, hi] = [ctx.inputs[0], ctx.inputs[1], ctx.inputs[2]];
    out[0] = lo + (x + 1.0) * 0.5 * (hi - lo);
}

fn tick_lerp(ctx: &mut TickCtx, out: &mut [f32]) {
    let [a, b, t] = [ctx.inputs[0], ctx.inputs[1], ctx.inputs[2]];
    out[0] = a + (b - a) * t;
}

fn tick_xfade(ctx: &mut TickCtx, out: &mut [f32]) {
    // Both legs through sin — cos(t·π/2) written as sin((1-t)·π/2) — so the emit needs only
    // the one sin shim and matches bit-for-bit. Same grouping as `emit_xfade`.
    let [a, b, t] = [ctx.inputs[0], ctx.inputs[1], ctx.inputs[2]];
    out[0] = a * ((1.0 - t) * std::f32::consts::FRAC_PI_2).sin()
        + b * (t * std::f32::consts::FRAC_PI_2).sin();
}

fn tick_uni(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0] * 0.5 + 0.5;
}

fn tick_bi(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0] * 2.0 - 1.0;
}

fn tick_pow(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].powf(ctx.inputs[1]);
}

fn tick_log(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].ln();
}

fn tick_log2(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].log2();
}

fn tick_log10(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].log10();
}

fn tick_mod(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = ctx.inputs[0].rem_euclid(ctx.inputs[1]);
}

fn tick_wrap(ctx: &mut TickCtx, out: &mut [f32]) {
    let [x, lo, hi] = [ctx.inputs[0], ctx.inputs[1], ctx.inputs[2]];
    out[0] = lo + (x - lo).rem_euclid(hi - lo);
}

fn tick_fold(ctx: &mut TickCtx, out: &mut [f32]) {
    // Reflect into [lo, hi]: wrap into one round-trip (2·range), then mirror the back half.
    // Ordered `>` (NaN ⇒ else branch), matching `emit_fold`'s `select_gt`.
    let [x, lo, hi] = [ctx.inputs[0], ctx.inputs[1], ctx.inputs[2]];
    let r = hi - lo;
    let t = (x - lo).rem_euclid(2.0 * r);
    let folded = if t > r { 2.0 * r - t } else { t };
    out[0] = lo + folded;
}

fn tick_linexp(ctx: &mut TickCtx, out: &mut [f32]) {
    let [x, ilo, ihi, olo, ohi] = [ctx.inputs[0], ctx.inputs[1], ctx.inputs[2], ctx.inputs[3], ctx.inputs[4]];
    out[0] = (ohi / olo).powf((x - ilo) / (ihi - ilo)) * olo;
}

fn tick_mtof(ctx: &mut TickCtx, out: &mut [f32]) {
    // powf(2, ·), not exp2 — the emit mirrors it through the shared pow shim.
    out[0] = 440.0 * 2f32.powf((ctx.inputs[0] - 69.0) / 12.0);
}

fn tick_ftom(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = 69.0 + 12.0 * (ctx.inputs[0] / 440.0).log2();
}

fn tick_dbamp(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = 10f32.powf(ctx.inputs[0] / 20.0);
}

fn tick_ampdb(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = 20.0 * ctx.inputs[0].log10();
}

// Ordered compares (NaN ⇒ 0.0) matching the emit's `select_*`; `!=` is unordered (NaN ⇒ 1.0)
// like Rust's `!=`, matching `emit_ne`'s inverted `select_eq`.
fn tick_lt(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = if ctx.inputs[0] < ctx.inputs[1] { 1.0 } else { 0.0 };
}

fn tick_gt(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = if ctx.inputs[0] > ctx.inputs[1] { 1.0 } else { 0.0 };
}

fn tick_le(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = if ctx.inputs[0] <= ctx.inputs[1] { 1.0 } else { 0.0 };
}

fn tick_ge(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = if ctx.inputs[0] >= ctx.inputs[1] { 1.0 } else { 0.0 };
}

fn tick_eq(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = if ctx.inputs[0] == ctx.inputs[1] { 1.0 } else { 0.0 };
}

fn tick_ne(ctx: &mut TickCtx, out: &mut [f32]) {
    out[0] = if ctx.inputs[0] != ctx.inputs[1] { 1.0 } else { 0.0 };
}
