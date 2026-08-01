//! The execution representation: a flat, topologically ordered program.
//!
//! A [`Program`] is what the audio thread runs, interpreted by [`crate::vm::Vm`].
//! Op `i` writes register `i`, and every input references a register with a
//! lower index (guaranteed by the topological order), so a single forward pass
//! evaluates the whole graph for one sample.

use crate::ugen::UGen;

/// The window the global sample clock wraps within before the pure f32 core sees it (see
/// [`Op::Now`]). A power of two `< 2^24`, so every value in `0..NOW_WINDOW` is an *exact*
/// integer in f32 — the windowed `now` never loses precision however long the engine runs
/// (integer add+mask, then one exact `u64 -> f32` cast). The trade-off is a hard ceiling on
/// any single `now`-relative interval: 2^23 samples ≈ 174.8 s at 48 kHz (≈ 87.4 s at 96 kHz).
/// The canonical clock the host owns is a full `u64` frame count; only this windowed view
/// reaches the DSP core.
pub const NOW_WINDOW: u64 = 1 << 23;

/// Reduce a `now`-relative difference back into the window, modularly — the single source of
/// truth for the wrap. Used by time UGens for `now - start`, which can straddle a wrap.
#[inline]
pub(crate) fn now_wrap(x: f32) -> f32 {
    x.rem_euclid(NOW_WINDOW as f32)
}

/// A per-sample scratch register. `Reg(i)` is written by op `i`.
#[derive(Clone, Copy, Debug)]
pub struct Reg(pub(crate) u32);

/// The two-input words the compiler lowers to [`Op::Bin`] instead of a UGen call: glue
/// arithmetic and scalar comparison are the most numerous node kinds in a patch, and matching
/// them inline in the interpreter loop skips the whole per-op call apparatus (input gather
/// through the arena, `TickCtx` slicing, the indirect `tick` call) for a single IEEE
/// expression. Each kind is bit-identical to its row's tick — the arm below carries the same
/// expression, in the same operand order, as the `tick_*` it replaces.
#[derive(Clone, Copy, Debug)]
pub enum BinKind {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
}

impl BinKind {
    /// The kind lowering `name`, if it is one of the inlined words.
    pub(crate) fn of(name: &str) -> Option<BinKind> {
        match name {
            "+" => Some(BinKind::Add),
            "-" => Some(BinKind::Sub),
            "*" => Some(BinKind::Mul),
            "/" => Some(BinKind::Div),
            "min" => Some(BinKind::Min),
            "max" => Some(BinKind::Max),
            "<" => Some(BinKind::Lt),
            ">" => Some(BinKind::Gt),
            "<=" => Some(BinKind::Le),
            ">=" => Some(BinKind::Ge),
            "==" => Some(BinKind::Eq),
            "!=" => Some(BinKind::Ne),
            _ => None,
        }
    }

    /// The row cost the inlined op replaces (see [`Program::weight`]): 1 unit for every kind
    /// but the division, which its row prices at 4. Keeping the cost identical to the row's
    /// keeps the host's install-time weight cap admitting exactly the same graphs.
    fn cost(self) -> u32 {
        match self {
            BinKind::Div => 4,
            _ => 1,
        }
    }
}

/// The one-input words the compiler lowers to [`Op::Un`] — the unary half of the inline fast
/// path (see [`BinKind`]). Each kind reproduces its row's tick expression exactly.
#[derive(Clone, Copy, Debug)]
pub enum UnKind {
    Abs,
    Neg,
    Uni,
    Bi,
    Floor,
}

impl UnKind {
    /// The kind lowering `name`, if it is one of the inlined words.
    pub(crate) fn of(name: &str) -> Option<UnKind> {
        match name {
            "abs" => Some(UnKind::Abs),
            "neg" => Some(UnKind::Neg),
            "uni" => Some(UnKind::Uni),
            "bi" => Some(UnKind::Bi),
            "floor" => Some(UnKind::Floor),
            _ => None,
        }
    }

    /// The row cost the inlined op replaces (see [`Program::weight`]).
    fn cost(self) -> u32 {
        match self {
            UnKind::Uni | UnKind::Bi => 2,
            UnKind::Abs | UnKind::Neg | UnKind::Floor => 1,
        }
    }
}

/// The three-input words the compiler lowers to [`Op::Tern`] — the ternary half of the inline
/// fast path (see [`BinKind`]). Each kind reproduces its row's tick expression exactly,
/// operand order included: `clip` is `x.max(lo).min(hi)`, never the mirrored form.
#[derive(Clone, Copy, Debug)]
pub enum TernKind {
    Clip,
    Lerp,
    Range,
}

impl TernKind {
    /// The kind lowering `name`, if it is one of the inlined words.
    pub(crate) fn of(name: &str) -> Option<TernKind> {
        match name {
            "clip" => Some(TernKind::Clip),
            "lerp" => Some(TernKind::Lerp),
            "range" => Some(TernKind::Range),
            _ => None,
        }
    }

    /// The row cost the inlined op replaces (see [`Program::weight`]).
    fn cost(self) -> u32 {
        match self {
            TernKind::Clip => 2,
            TernKind::Lerp => 3,
            TernKind::Range => 4,
        }
    }
}

/// A single operation. `Const` is the only leaf carrying an immediate; every other op
/// is a uniform UGen invocation. Its `input_count` inputs are `inputs[input_start..]` in the
/// program's flat input arena, its `state_slots` persistent slots start at `state_base`, and
/// its `buffer_len` sample-memory f32s start at `buffer_base` in the buffer arena. `input_count`
/// is stored (not read from the row) because a variadic generator's arity is fixed at
/// graph-construction, not by its row; `state_slots`/`outputs` still come from the row.
#[derive(Clone, Copy, Debug)]
pub enum Op {
    Const(f32),
    Ugen {
        /// The generator's row, resolved once at compile time: the hot loop reads
        /// `tick`/`outputs`/`state_slots` straight off this reference instead of going
        /// through the global `UGENS` table, whose `LazyLock` costs an atomic load per
        /// access — per op, per sample, on the audio thread.
        def: &'static UGen,
        input_start: u32,
        input_count: u32,
        state_base: u32,
        buffer_base: u32,
        /// This instance's sample-memory length (f32s) at `buffer_base`. Per-op (not read from
        /// the row) so a buffer can be sized at graph-construction and shared: an anonymous
        /// buffer (e.g. `delay`) takes the row's `buffer_len`, a named buffer takes its
        /// declared length, and several ops can point at one named region.
        buffer_len: u32,
    },
    /// An inlined two-input word (see [`BinKind`]): `regs[a] op regs[b]`, straight
    /// in the interpreter loop — no input arena, no state, no tick call.
    Bin {
        kind: BinKind,
        a: Reg,
        b: Reg,
    },
    /// An inlined one-input word (see [`UnKind`]): `op regs[a]`, straight in the
    /// interpreter loop — no input arena, no state, no tick call.
    Un {
        kind: UnKind,
        a: Reg,
    },
    /// An inlined three-input word (see [`TernKind`]): `op(regs[a], regs[b], regs[c])`,
    /// straight in the interpreter loop — no input arena, no state, no tick call.
    Tern {
        kind: TernKind,
        a: Reg,
        b: Reg,
        c: Reg,
    },
    /// Read a feedback bus: load `buses[slot]` (the value stored last sample). A leaf, so it
    /// can sit anywhere in topological order and breaks the feedback cycle. Buses live in
    /// their own arena (separate from per-UGen state), so `slot` is independent of UGen state
    /// layout — adding or removing a bus never shifts a UGen's `state_base`.
    FbRead {
        slot: u32,
    },
    /// Read audio input `channel` (the current frame's sample), supplied by the host. A
    /// leaf; `channel` is `< in_channels` by construction, so the read is always in range.
    Input {
        channel: u32,
    },
    /// Read control `lane` from the host's per-block control plane (a host-supplied value:
    /// the note's gate/notefreq/vel, the transport tempo, or a named parameter). A leaf, like
    /// [`Op::Input`], but frame-invariant: the plane is latched once per block, so the value is
    /// constant across the block. `lane` is `< control_len` by construction.
    Control {
        lane: u32,
    },
    /// Read the global sample clock as a signal: the current frame's absolute position,
    /// reduced into [`NOW_WINDOW`] (`(block_start_pos + frame) & (NOW_WINDOW - 1)`). A leaf,
    /// like [`Op::Input`] but supplied by the *executor*, not the host — frame-strided, so it
    /// advances one per sample within a block. This is the pure-core face of time: a UGen reads
    /// it as an input register (or ambiently via `TickCtx`), never as a global.
    Now,
}

/// A feedback write-back: after each frame, store register `source` into `buses[slot]`
/// so the matching [`Op::FbRead`] reads it next sample (the one-sample delay).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Feedback {
    pub(crate) slot: u32,
    pub(crate) source: Reg,
}

/// A compiled audio graph, ready to run.
#[derive(Debug)]
pub struct Program {
    /// Ops in topological order. Each op writes a contiguous block of registers (one for a
    /// leaf, the UGen's `outputs` for a generator), assigned by the compiler in op order — so
    /// every input still references a strictly-lower register index (the topological invariant).
    pub(crate) ops: Vec<Op>,
    /// Total registers the ops write — the per-sample scratch size. Equals `ops.len()` while
    /// every op is single-output; a multi-output op writes several, decoupling the register
    /// space from the op count.
    pub(crate) register_count: usize,
    /// Flat arena of op inputs. An `Op::Ugen`'s inputs are the `arity` registers at
    /// `inputs[input_start..]`. Separated from `ops` so an op stays small and `Copy`.
    pub(crate) inputs: Vec<Reg>,
    /// Number of persistent per-UGen state slots the program needs (oscillator phase, filter
    /// memory, …). Independent of the bus count — buses live in their own plane.
    pub(crate) state_len: usize,
    /// Sparse fresh-init values for the state plane: `(slot, value)` pairs applied over the
    /// all-zero fill when a [`crate::vm::Vm`] is built or reset. Today only noise sources use
    /// it — the compiler seeds each one's sample-counter slot so co-existing instances
    /// decorrelate (see [`crate::ugen::noise_seed`]). Empty when no op is seeded.
    pub(crate) initial_state: Vec<(u32, f32)>,
    /// Number of feedback-bus slots the program needs, in their own arena (one per declared
    /// bus). Kept separate from `state_len` so adding/removing a bus never renumbers state.
    pub(crate) bus_len: usize,
    /// Total sample-memory the program needs (sum of every UGen's `buffer_len`), as one
    /// flat arena; an `Op::Ugen`'s slice starts at its `buffer_base`.
    pub(crate) buffer_len: usize,
    /// Audio input channels the program reads (max `Op::Input` channel + 1, or 0 if none).
    /// The host supplies an input block this wide per frame, so every `Input` is in range.
    pub(crate) in_channels: usize,
    /// Control-plane width the program reads (max `Op::Control` lane + 1, or 0 if none). The
    /// host supplies a control block this wide, latched once per block, so every `Control` is
    /// in range. Per-voice lanes and the transport lane live here (see [`crate::graph`]).
    pub(crate) control_len: usize,
    /// The registers feeding the output channels, in channel order. Every valid program has
    /// at least one channel (the front-end requires `out`).
    pub(crate) outputs: Vec<Reg>,
    /// Declared named parameters `(name, default)` in declaration order; entry `i` owns control
    /// lane `PARAM_BASE + i`. Carried whether or not the graph references them, so the host can
    /// resolve names and fill defaults from the program alone.
    pub(crate) params: Vec<(String, f32)>,
    /// End-of-frame feedback write-backs (the source side of each one-sample delay).
    pub(crate) feedbacks: Vec<Feedback>,
    /// Sample rate the program was compiled for (Hz).
    pub(crate) sample_rate: f32,
}

impl Program {
    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    /// The flat input arena; an `Op::Ugen` indexes it by `input_start`.
    pub(crate) fn inputs(&self) -> &[Reg] {
        &self.inputs
    }

    /// Size of the per-sample register scratch (the total registers the ops write).
    pub fn num_registers(&self) -> usize {
        self.register_count
    }

    pub fn state_len(&self) -> usize {
        self.state_len
    }

    /// Sparse fresh-init for the state plane (see the field): `(slot, value)` pairs the VM
    /// applies after zeroing its state arena.
    pub(crate) fn initial_state(&self) -> &[(u32, f32)] {
        &self.initial_state
    }

    /// Number of feedback-bus slots (the bus arena's size), separate from `state_len`.
    pub fn bus_len(&self) -> usize {
        self.bus_len
    }

    /// Total sample-memory the program needs, as one flat f32 arena.
    pub fn buffer_len(&self) -> usize {
        self.buffer_len
    }

    /// Audio input channels the program reads; the host's input block is this wide per frame.
    pub fn in_channels(&self) -> usize {
        self.in_channels
    }

    /// Control-plane width the program reads; the host's control block is this wide, latched
    /// once per block.
    pub fn control_len(&self) -> usize {
        self.control_len
    }

    /// The registers feeding the output channels, in channel order. The VM writes every one
    /// of these into the render block.
    pub fn outputs(&self) -> &[Reg] {
        &self.outputs
    }

    /// Audio channels routed to the device — the width of [`outputs`](Self::outputs).
    pub fn audio_channels(&self) -> usize {
        self.outputs.len()
    }

    /// Declared named parameters `(name, default)`, in declaration (= lane) order.
    pub fn params(&self) -> &[(String, f32)] {
        &self.params
    }

    /// The control lane of declared parameter `name`, if the program declares it.
    pub fn param_lane(&self, name: &str) -> Option<u32> {
        self.params
            .iter()
            .position(|(n, _)| n == name)
            .map(|i| (crate::graph::PARAM_BASE + i) as u32)
    }

    /// The feedback write-backs applied at the end of each frame.
    pub(crate) fn feedbacks(&self) -> &[Feedback] {
        &self.feedbacks
    }

    /// The summed per-sample cost of the program: Σ over its ops of the producing UGen's
    /// [`cost`](crate::ugen::UGen::cost), counting each non-UGen leaf (`Const`, a
    /// bus/input/control/clock read) as one unit. A machine-independent estimate of how much
    /// arithmetic one frame costs — it ranks two patches identically on any CPU.
    pub fn weight(&self) -> u32 {
        self.ops
            .iter()
            .map(|op| match op {
                Op::Ugen { def, .. } => def.cost as u32,
                Op::Bin { kind, .. } => kind.cost(),
                Op::Un { kind, .. } => kind.cost(),
                Op::Tern { kind, .. } => kind.cost(),
                _ => 1,
            })
            .sum()
    }
}
