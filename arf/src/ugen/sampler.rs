//! Players/recorders over named buffers: `play`, `record`. Each holds its own head (one state
//! slot) and reads/writes a shared, named, variable-length buffer region — so a `record`/`play`
//! pair on one buffer is a looper, and several `play`s tap one recording. The buffer length is
//! non-power-of-two in general (sized to the request), so instead of a mask the head is
//! clamped into range with a `min` and re-wrapped by compare on advance — no `%` division.

use super::{Arity, Category, TickCtx, UGen, signal};

pub(super) static UGENS: &[UGen] = &[
    // play ( -- sig )   read a named buffer at the play head, advancing & looping
    UGen {
        name: "play",
        category: Category::Buffer,
        description: "Plays back a named buffer, looping at the play head. With a `~slot` name before it (`~bass play`) it is the session directive that starts the slot instead — see `stop`/`fade`.",
        examples: &[
            "buf a 2  noise 0.2 * record a  drop  play a  out",
            "buf a 1  220 saw 0.2 * record a  drop  play a  600 lpf 0.3 * out",
        ],
        arity: Arity::Fixed(0),
        inputs: &[],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 4,
        tick: tick_play,
    },
    // record ( in -- in )   write `in` to a named buffer at the write head (advancing &
    // looping), passing it through so the live signal can also be heard or chained
    UGen {
        name: "record",
        category: Category::Buffer,
        description: "Records the input into a named buffer (looping), passing the signal through.",
        examples: &[
            "220 sine 0.2 * record a  out",
            "buf a 2  noise 0.2 * record a  drop  play a  out",
        ],
        arity: Arity::Fixed(1),
        inputs: &[signal("in")],
        outputs: 1,
        state_slots: 1,
        buffer_len: 0,
        cost: 4,
        tick: tick_record,
    },
];

fn tick_play(ctx: &mut TickCtx, out: &mut [f32]) {
    let len = ctx.buffer.len();
    // A `play` always names a buffer (≥ 1 sample); guard anyway so an unnamed misuse is silent.
    if len == 0 {
        out[0] = 0.0;
        return;
    }
    // Heads stay < len by construction (the advance re-wraps); `min` keeps a corrupt slot
    // from panicking the audio thread without paying `%`'s division.
    let head = (ctx.state[0] as usize).min(len - 1);
    out[0] = ctx.buffer[head];
    let next = head + 1;
    ctx.state[0] = (if next >= len { 0 } else { next }) as f32;
}

fn tick_record(ctx: &mut TickCtx, out: &mut [f32]) {
    let x = ctx.inputs[0];
    let len = ctx.buffer.len();
    if len != 0 {
        let head = (ctx.state[0] as usize).min(len - 1);
        ctx.buffer[head] = x;
        let next = head + 1;
        ctx.state[0] = (if next >= len { 0 } else { next }) as f32;
    }
    out[0] = x; // passthrough
}
