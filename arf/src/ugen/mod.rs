//! The UGen contract and the assembled vocabulary.
//!
//! A UGen is declared once — its name (the word a front-end builds it by), input arity,
//! persistent state, and per-sample DSP — and graph front-ends, the compiler
//! ([`crate::compile`]),
//! and the VM ([`crate::vm`]) all read the one [`UGENS`] table. The *contract* (this struct
//! family, [`lookup`]/[`def`], and the well-formedness tests) lives here; the generators
//! themselves live in category submodules ([`math`], [`osc`], [`filter`], …), each owning its
//! rows and their `tick` bodies. [`UGENS`] concatenates those per-module slices into one
//! canonical list, so a `UGenId` stays a flat index and nothing downstream changes.
//!
//! Adding a generator is adding one row (and its `tick`) to the right category file;
//! adding a category is `mod foo;` plus one line in the [`UGENS`] assembly.

use std::sync::LazyLock;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

mod buffer;
mod dynamics;
mod filter;
mod math;
mod osc;
mod pan;
mod reverb;
mod sampler;
mod shaper;
mod source;
mod time;

// The compiler reaches these to seed each noise source's counter slot for per-instance
// decorrelation. Needed on every target (the browser compiles patches too).
pub(crate) use source::{noise_seed, seed_slot};

/// The physical meaning of a UGen input, for the front-end's dictionary and hovers. A signal
/// input with no specific unit is [`Unit::None`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Hz,
    Seconds,
    Amplitude,
    Ratio,
    None,
}

/// The classical UGen family a generator belongs to — groups the front-end's dictionary
/// (cagire's patch-word sidebar). A semantic taxonomy (à la SuperCollider/Csound),
/// deliberately independent of the `src/ugen/` module layout: the `time` module, for
/// instance, supplies both [`Category::Trigger`] and [`Category::Envelope`]. New families
/// are added as variants when their UGens land.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Oscillator,
    Noise,
    Filter,
    Distortion,
    Dynamics,
    Delay,
    Envelope,
    Trigger,
    Panner,
    Buffer,
    Math,
}

impl Category {
    /// Every category, in signal-flow display order (generators → processors → control →
    /// routing → math) — the order the front-end's dictionary groups by.
    pub const ALL: [Category; 11] = [
        Category::Oscillator,
        Category::Noise,
        Category::Filter,
        Category::Distortion,
        Category::Dynamics,
        Category::Delay,
        Category::Envelope,
        Category::Trigger,
        Category::Panner,
        Category::Buffer,
        Category::Math,
    ];

    /// The display name for the front-end's dictionary.
    pub fn label(self) -> &'static str {
        match self {
            Category::Oscillator => "Oscillator",
            Category::Noise => "Noise",
            Category::Filter => "Filter",
            Category::Distortion => "Distortion",
            Category::Dynamics => "Dynamics",
            Category::Delay => "Delay",
            Category::Envelope => "Envelope",
            Category::Trigger => "Trigger",
            Category::Panner => "Panner",
            Category::Buffer => "Buffer",
            Category::Math => "Math",
        }
    }
}

/// A self-describing declaration of one UGen input: its name, unit, and a sensible `range`
/// and `default`. Declared once in the [`UGENS`] table so the parser and the front-end's
/// dictionary read the same source of truth.
#[derive(Clone, Copy, Debug)]
pub struct InputDescriptor {
    pub name: &'static str,
    pub unit: Unit,
    pub range: (f32, f32),
    pub default: f32,
}

/// A signal input carrying no specific unit (e.g. the thing fed into `lpf`/`delay`/`+`).
/// Private to the `ugen` module tree; the category submodules reach it via `super::signal`.
const fn signal(name: &'static str) -> InputDescriptor {
    InputDescriptor { name, unit: Unit::None, range: (-1.0, 1.0), default: 0.0 }
}

/// A handle to a UGen definition: an index into [`UGENS`]. Only this module mints one
/// (via [`lookup`]), so a `UGenId` always refers to a real row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UGenId(pub(crate) u32);

/// The largest `arity` any row declares. Bounds the VM's per-op input scratch so it
/// needs no allocation; [`tests::table_is_well_formed`] checks the table against it. Five covers
/// `adsr` (gate + attack/decay/sustain/release).
pub const MAX_ARITY: usize = 5;

/// The largest `outputs` any row declares. Bounds the VM's per-op output scratch so a
/// multi-output tick needs no allocation; [`tests::table_is_well_formed`] checks the table
/// against it. Four covers the state-variable filter's lp/bp/hp/notch taps.
pub const MAX_OUTPUTS: usize = 4;

/// How a generator consumes its operands at graph-construction. `Fixed(n)` pops `n` operands
/// and broadcasts across channels (SuperCollider-style multichannel expansion); `Variadic`
/// consumes one whole channel-list as its inputs, so the arity is the list's width (e.g.
/// `mix`); `VariadicLed` is `Variadic` plus a leading mono operand made input 0 (the triggered
/// list-walkers `seq`/`select`/`linseg`). A given instance's actual input count is recorded on
/// its [`crate::ir::Op`] (`input_count`), bounded by [`crate::graph::MAX_CHANNELS`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arity {
    Fixed(usize),
    Variadic,
    /// `Variadic` with a leading mono operand: it is popped from *below* the channel-list and
    /// becomes input 0, the list becoming inputs 1.. . Backs the triggered list-walkers, whose
    /// `tick` reads `inputs[0]` as the trigger/index; `shape` constrains the list length.
    VariadicLed { shape: ListShape },
}

/// The list-length law a [`Arity::VariadicLed`] generator imposes on its channel-list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListShape {
    /// Any non-empty list (`seq`, `select`).
    Any,
    /// Odd length ≥ 3 — a start level then (time, level) pairs (`linseg`).
    OddAtLeastThree,
}

/// The per-op execution context handed to a UGen's [`UGen::tick`]: its gathered `inputs`,
/// its persistent `state` slice, its sample-memory `buffer`, the sample rate, and the sample
/// clock — the single view onto the engine's memory planes. Built fresh per op by the VM —
/// no allocation.
pub struct TickCtx<'a> {
    /// The UGen's signal inputs (`arity` of them), gathered from registers.
    pub inputs: &'a [f32],
    /// The UGen's persistent state slots (`state_slots` of them).
    pub state: &'a mut [f32],
    /// The UGen's sample-memory buffer (`buffer_len` f32s, a power of two), for delay
    /// lines and wavetables. Empty for UGens that declare no buffer. Index it masked
    /// (`i & (len - 1)`) so every access is in range.
    pub buffer: &'a mut [f32],
    /// The sample rate in Hz.
    pub sr: f32,
    /// The global sample clock for this frame: the current absolute frame position reduced
    /// into [`crate::ir::NOW_WINDOW`]. Supplied by the executor (not a global read), exactly
    /// like `sr` — time-based UGens read it ambiently. Advances one per sample within a block.
    pub now: f32,
}

impl TickCtx<'_> {
    /// Read the op's buffer at a fractional position, linearly interpolating between the two
    /// neighboring samples. Indices are masked like every buffer access, so any `pos ≥ 0` is
    /// in range. Owned by the context (not the UGens) so every executor mirrors one
    /// definition.
    pub fn load_lerp(&self, pos: f32) -> f32 {
        let mask = self.buffer.len() - 1;
        let i = pos as usize;
        let frac = pos - i as f32;
        let a = self.buffer[i & mask];
        let b = self.buffer[(i + 1) & mask];
        a + frac * (b - a)
    }
}

/// One unit generator, declared once and read by parser, compiler, and VM.
#[derive(Clone)]
pub struct UGen {
    /// The Forth word that builds it.
    pub name: &'static str,
    /// The classical family this generator belongs to — groups it in the front-end's
    /// dictionary. Independent of the module it lives in.
    pub category: Category,
    /// A one-line description of what the generator does — the prose the signature can't
    /// carry, surfaced in the front-end's dictionary and hovers.
    pub description: &'static str,
    /// 1–3 complete, runnable Forth programs demonstrating the generator — each ends in `out`.
    pub examples: &'static [&'static str],
    /// How the parser consumes operands: `Fixed(n)` pops `n` (and `inputs.len() == n`);
    /// `Variadic` consumes one channel-list (and `inputs` describes a single representative
    /// element). Checked by [`tests::table_is_well_formed`].
    pub arity: Arity,
    /// Per-input descriptors (name, unit, range, default) — the documented signature as
    /// data, read by the parser and the front-end's dictionary.
    pub inputs: &'static [InputDescriptor],
    /// Number of output signals (≥ 1). Most generators produce one; a multi-output generator
    /// (e.g. `svf`'s lp/bp/hp/notch taps, or `pan2`'s L/R) produces several from one shared
    /// computation, each landing in its own register — selected downstream by an output port.
    pub outputs: usize,
    /// Persistent f32 state slots (oscillator phase, filter memory, …).
    pub state_slots: usize,
    /// Sample-memory buffer length in f32s — a power of two, or 0 for no buffer. Delay
    /// lines and wavetables declare one; allocated when the engine is built, never on the
    /// audio thread. Power-of-two so the context can mask indices into range cheaply.
    pub buffer_len: usize,
    /// Relative per-sample compute cost, in *cost units* (≈ one float op = 1 unit): the
    /// `tick`'s arithmetic counted by inspection, with transcendentals (`sin`/`exp`/`tanh`/
    /// `pow`/`log`) ≈ 10, `sqrt`/`div`/`rem` ≈ 4, a buffer `load_lerp` ≈ 3, and `+1` per state
    /// slot touched. A machine-independent estimate, never a measured time — summed over a
    /// program's ops it yields the *graph weight* ([`crate::ir::Program::weight`]), a
    /// reproducible complexity figure that ranks patches identically on any CPU. Metadata
    /// only: it never enters `tick`, so it cannot change the sound.
    pub cost: u16,
    /// The per-sample DSP: read the context's inputs/state/sr, write `out[0..outputs]`.
    pub tick: fn(&mut TickCtx<'_>, out: &mut [f32]),
}

/// The whole UGen vocabulary, assembled from the category modules. The order here is the
/// canonical row order (and therefore the `UGenId` order); it is deterministic and stable.
pub static UGENS: LazyLock<Vec<UGen>> = LazyLock::new(|| {
    [
        math::UGENS,
        shaper::UGENS,
        osc::UGENS,
        filter::UGENS,
        dynamics::UGENS,
        pan::UGENS,
        buffer::UGENS,
        reverb::UGENS,
        sampler::UGENS,
        source::UGENS,
        time::UGENS,
    ]
    .concat()
});

/// Resolve a UGen word to its id, or `None` if it names no generator. Public so any
/// graph front-end (cagire's `arf-forth`, or any other language) can build nodes by name.
pub fn lookup(name: &str) -> Option<UGenId> {
    UGENS
        .iter()
        .position(|u| u.name == name)
        .map(|i| UGenId(i as u32))
}

/// The definition behind a [`UGenId`].
pub fn def(id: UGenId) -> &'static UGen {
    &UGENS[id.0 as usize]
}

// A `UGenId` is a flat index into `UGENS`, which is stable only within one arf build. The
// serialized graph is the cross-crate patch boundary (a host may deserialize with a
// different arf instance), so a ugen crosses it *by name* — resolved back to an index on
// the far side. Version-robust: reordering the table cannot silently remap a patch.
impl Serialize for UGenId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(def(*self).name)
    }
}

impl<'de> Deserialize<'de> for UGenId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        lookup(&name).ok_or_else(|| de::Error::custom(format!("unknown ugen {name:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_well_formed() {
        for u in UGENS.iter() {
            match u.arity {
                // A fixed row's descriptors are the documented signature as data: one per input.
                Arity::Fixed(n) => {
                    assert!(n <= MAX_ARITY, "`{}` arity {n} exceeds MAX_ARITY {MAX_ARITY}", u.name);
                    assert_eq!(
                        u.inputs.len(),
                        n,
                        "`{}` has {} input descriptors but arity {n}",
                        u.name,
                        u.inputs.len()
                    );
                }
                // A variadic row describes one representative element; its width is per-call.
                Arity::Variadic => assert!(
                    !u.inputs.is_empty(),
                    "`{}` (variadic) needs a per-element input descriptor",
                    u.name
                ),
                // A led-variadic row describes input 0 (its leading trigger/index operand).
                Arity::VariadicLed { .. } => assert!(
                    !u.inputs.is_empty(),
                    "`{}` (led-variadic) needs an input descriptor for its leading operand",
                    u.name
                ),
            }
            assert!(
                (1..=MAX_OUTPUTS).contains(&u.outputs),
                "`{}` has {} outputs (must be 1..={MAX_OUTPUTS})",
                u.name,
                u.outputs
            );
            // Every row carries a positive cost weight — leaves alone are free; a generator
            // does at least one float op, and `graph_weight` relies on this lower bound.
            assert!(u.cost >= 1, "`{}` must declare a cost >= 1", u.name);
            // Every row carries a one-line description — the prose surfaced everywhere.
            assert!(!u.description.is_empty(), "`{}` has no description", u.name);
            assert!(
                !u.description.contains('\n'),
                "`{}` description must be a single line",
                u.name
            );
            // Every row carries 1–3 runnable examples — the demos the front-end's dictionary
            // surfaces. The `out` check is a cheap "is a full program" guard; the real
            // runnability proof is `ugen_examples_evaluate` in cagire's
            // `crates/arf-forth/tests/harness.rs`.
            assert!(!u.examples.is_empty(), "`{}` has no examples", u.name);
            assert!(u.examples.len() <= 3, "`{}` has {} examples (max 3)", u.name, u.examples.len());
            for ex in u.examples {
                assert!(!ex.trim().is_empty(), "`{}` has an empty example", u.name);
                assert!(
                    ex.contains("out"),
                    "`{}` example must be a full program (contain `out`): {ex}",
                    u.name
                );
            }
            for d in u.inputs {
                assert!(
                    d.range.0 <= d.range.1,
                    "`{}` input `{}` has an inverted range {:?}",
                    u.name,
                    d.name,
                    d.range
                );
            }
        }
        for (i, a) in UGENS.iter().enumerate() {
            for b in &UGENS[i + 1..] {
                assert_ne!(a.name, b.name, "duplicate UGen name `{}`", a.name);
            }
            // a name resolves back to its own row
            assert_eq!(lookup(a.name).map(|id| id.0 as usize), Some(i));
        }
        assert_eq!(lookup("nonesuch"), None);
    }
}
