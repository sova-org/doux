//! Dynamics & analysis: `follow` (amplitude follower) and `comp` (compressor). Both ride
//! the same asymmetric one-pole envelope — `lag`'s coefficient with separate attack and
//! release time constants, picked per sample by whether the rectified input is rising.

use super::{signal, Arity, Category, InputDescriptor, Rate, TickCtx, UGen, Unit};

pub(super) static UGENS: &[UGen] = &[
    // follow ( in attack release -- env )  state: [env]   rectify-and-smooth amplitude follower
    UGen { name: "follow", category: Category::Dynamics, description: "Amplitude follower — tracks |in| with separate attack/release smoothing (a gate composes: `… follow 0.2 >`).",
           examples: &["440 sine 0.5 *  0.01 0.1 follow  out", "noise 0.3 *  0.001 0.05 follow  440 sine *  0.4 * out"], arity: Arity::Fixed(3),
           inputs: &[signal("in"),
                     InputDescriptor { name: "attack", unit: Unit::Seconds, range: (0.0, 1.0), default: 0.01, rate: Rate::Audio },
                     InputDescriptor { name: "release", unit: Unit::Seconds, range: (0.0, 5.0), default: 0.1, rate: Rate::Audio }],
           outputs: 1, state_slots: 1, buffer_len: 0, rate: Rate::Audio, cost: 22, tick: tick_follow },
    // comp ( in thresh ratio attack release -- sig )  state: [env]   feed-forward compressor
    UGen { name: "comp", category: Category::Dynamics, description: "Compressor — reduces gain by `ratio` while the followed level exceeds `thresh`.",
           examples: &["440 sine 0.8 *  0.3 4 0.01 0.1 comp  out", "noise 0.2 8 0.005 0.2 comp  0.5 * out"], arity: Arity::Fixed(5),
           inputs: &[signal("in"),
                     InputDescriptor { name: "thresh", unit: Unit::Amplitude, range: (0.0, 1.0), default: 0.3, rate: Rate::Audio },
                     InputDescriptor { name: "ratio", unit: Unit::Ratio, range: (1.0, 20.0), default: 4.0, rate: Rate::Audio },
                     InputDescriptor { name: "attack", unit: Unit::Seconds, range: (0.0, 1.0), default: 0.01, rate: Rate::Audio },
                     InputDescriptor { name: "release", unit: Unit::Seconds, range: (0.0, 5.0), default: 0.1, rate: Rate::Audio }],
           outputs: 1, state_slots: 1, buffer_len: 0, rate: Rate::Audio, cost: 32, tick: tick_comp },
];

/// One step of the shared asymmetric follower: smooth `|x|` into `env` with `lag`'s
/// coefficient `a = 1 − e^{−1/(t·sr)}`, choosing the attack constant while rising
/// (`|x| > env`) and the release constant while falling. Both ticks call this so the
/// envelope math lives here once; `emit_follow_env` mirrors it op-for-op.
fn follow_env(env: f32, x: f32, attack: f32, release: f32, sr: f32) -> f32 {
    let xa = x.abs();
    let atk = attack.max(0.0);
    let rel = release.max(0.0);
    let aa = (1.0 - (-1.0 / (atk * sr)).exp()).clamp(0.0, 1.0);
    let ar = (1.0 - (-1.0 / (rel * sr)).exp()).clamp(0.0, 1.0);
    let a = if xa > env { aa } else { ar };
    env + a * (xa - env)
}

fn tick_follow(ctx: &mut TickCtx, out: &mut [f32]) {
    let env = follow_env(ctx.state[0], ctx.inputs[0], ctx.inputs[1], ctx.inputs[2], ctx.sr);
    ctx.state[0] = env;
    out[0] = env;
}

fn tick_comp(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    let env = follow_env(ctx.state[0], x, ctx.inputs[3], ctx.inputs[4], ctx.sr);
    ctx.state[0] = env;
    // Feed-forward gain: above `thresh` the output level follows t·(env/t)^{1/ratio}, i.e. a
    // gain of (env/t)^{1/ratio − 1}; below it, unity. The floors keep the divide and the
    // pow finite for degenerate thresh/ratio (and de-NaN them, like every coefficient path).
    let t = ctx.inputs[1].max(0.001);
    let r = ctx.inputs[2].max(1.0);
    let gain = if env > t { (env / t).powf(1.0 / r - 1.0) } else { 1.0 };
    out[0] = x * gain;
}
