//! State reconciliation across a hot-swap.
//!
//! Editing a live patch builds a fresh [`Program`] whose state buffer starts at zero.
//! Without reconciliation every edit restarts oscillator phases and clears filter
//! memory — an audible click. This module decides which stateful ops of the new program
//! *correspond* to ops of the old one, so their persistent state can be carried over.
//!
//! Correspondence between two arbitrary graphs is underdetermined (there is no ground
//! truth for "the same node after an edit"), so we use a deterministic key:
//!
//! `key = (structural signature, ordinal among same-signature stateful ops)`
//!
//! The signature is a Merkle-style fold over an op's upstream cone, with `Const` leaves
//! contributing a fixed tag — their *value* is ignored, because a constant feeding a
//! node is a parameter, not identity. So tweaking a frequency (`440 sine` → `441 sine`)
//! keeps the phase, while changing the generator kind or the upstream structure resets
//! it. Identical siblings are told apart by their ordinal in op order.
//!
//! The plan is built on the control thread and applied on the audio thread at adoption
//! (a bounded list of slice copies — no allocation, no hashing in the callback).

use crate::ir::{Op, Program};
use crate::ugen;

/// Which persistent arena a stateful node — and its migration — lives in. Per-UGen state and
/// feedback buses are separate arenas, so a migration must target the right one. (Buffers are
/// donated whole, not copied entry-by-entry, so they are not a migration plane.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plane {
    State,
    Bus,
}

/// A stateful op of a program: its structural signature, which plane its persistent slots
/// live in, and where (the base offset within that plane). Produced in op order so
/// equal-signature nodes get a stable ordinal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatefulNode {
    pub sig: u64,
    pub plane: Plane,
    pub base: u32,
    pub slots: u32,
}

/// One migration: copy `slots` f32s from the old arena's `old_base` to the new arena's
/// `new_base`, within `plane`'s arena (state or bus).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Migrate {
    pub plane: Plane,
    pub old_base: u32,
    pub new_base: u32,
    pub slots: u32,
}

/// One buffer-bearing op's identity for whole-arena donation: its structural signature and
/// buffer length, in op (and thus arena) order. Two programs can donate buffer contents iff
/// these lists match — then every op's buffer region lines up at the same offset, so swapping
/// the flat arena lands each delay line's contents exactly where the new program expects them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferSlot {
    pub sig: u64,
    pub len: u32,
}

/// The structural signatures of a program, in one forward pass: `op_sig[i]` is op `i`'s
/// identity as a stateful/buffer-bearing entity, and `reg_sig[r]` is register `r`'s identity
/// as a *value* — its producing op's signature folded with the output port, so the two taps of
/// a multi-output op read as distinct upstream values. Op `i` reads only lower-indexed
/// registers, so the pass fills both in dependency order. [`signatures`] keys state on
/// `op_sig` and a feedback source on `reg_sig`; [`buffer_layout`] keys buffers on `op_sig` —
/// so all see the same notion of "the same node after an edit".
fn op_signatures(program: &Program) -> (Vec<u64>, Vec<u64>) {
    let arena = program.inputs();
    let ops = program.ops();
    let mut op_sig = vec![0_u64; ops.len()];
    let mut reg_sig = vec![0_u64; program.num_registers()];
    let mut reg_cursor = 0usize;
    for (i, op) in ops.iter().enumerate() {
        let (s, outputs) = match *op {
            Op::Const(_) => (hash(CONST_TAG, &[]), 1),
            // A feedback read is a stable leaf, like `Const`: its identity does not depend
            // on the bus's slot index, so adding or removing an unrelated bus leaves every
            // downstream node's signature unchanged. The bus's *value* migration is keyed
            // separately, by the structure feeding it (see `signatures`).
            Op::FbRead { .. } => (hash(FBREAD_TAG, &[]), 1),
            // An input read is a stable leaf keyed by its channel, distinct from a
            // constant feeding the same consumer (so `in sine` and `440 sine` differ).
            Op::Input { channel } => (hash(INPUT_TAG, &[channel as u64]), 1),
            // A control read is a stable leaf keyed by its lane — so two replicated voices
            // (identical except for their lane) get distinct signatures and migrate to
            // *their own* voice across a hot-swap instead of swapping phases.
            Op::Control { lane } => (hash(CONTROL_TAG, &[lane as u64]), 1),
            // The sample clock is a stable leaf, like `FbRead`: one global clock, so every
            // `now` reads identically and folds the same tag into its consumers' signatures.
            Op::Now => (hash(NOW_TAG, &[]), 1),
            Op::Ugen { ugen, input_start, input_count, .. } => {
                let def = ugen::def(ugen);
                let start = input_start as usize;
                let n_in = input_count as usize;
                // Fold the input count, then each input's register signature: two ops of the
                // same kind but different (variadic) widths read as structurally distinct.
                let mut parts = Vec::with_capacity(n_in + 2);
                parts.push(ugen.0 as u64);
                parts.push(n_in as u64);
                for k in 0..n_in {
                    parts.push(reg_sig[arena[start + k].0 as usize]);
                }
                (hash(UGEN_TAG, &parts), def.outputs)
            }
        };
        op_sig[i] = s;
        // Each output register folds in its port, so a consumer of port 1 of a multi-output
        // op differs from a consumer of port 0 even though they share the one op signature.
        for port in 0..outputs {
            reg_sig[reg_cursor + port] = hash(OUTPUT_TAG, &[s, port as u64]);
        }
        reg_cursor += outputs;
    }
    (op_sig, reg_sig)
}

/// The stateful ops of `program`, each tagged with its plane and structural signature, in op
/// order (so equal-signature nodes get a stable ordinal).
pub fn signatures(program: &Program) -> Vec<StatefulNode> {
    let (op_sig, reg_sig) = op_signatures(program);
    let mut out = Vec::new();
    for (i, op) in program.ops().iter().enumerate() {
        if let Op::Ugen { ugen, state_base, .. } = *op {
            let def = ugen::def(ugen);
            if def.state_slots > 0 {
                out.push(StatefulNode {
                    sig: op_sig[i],
                    plane: Plane::State,
                    base: state_base,
                    slots: def.state_slots as u32,
                });
            }
        }
    }
    // Each feedback bus is stateful too: key it by the structure of what feeds it (the source
    // register), so an unchanged feedback path keeps its stored sample across an edit.
    for fb in program.feedbacks() {
        out.push(StatefulNode {
            sig: hash(FB_TAG, &[reg_sig[fb.source.0 as usize]]),
            plane: Plane::Bus,
            base: fb.slot,
            slots: 1,
        });
    }
    out
}

/// The buffer-bearing ops of `program`, by `(signature, len)`, in op order. Used to decide
/// whether the whole buffer arena can be donated across a hot-swap (see [`can_donate`]).
pub fn buffer_layout(program: &Program) -> Vec<BufferSlot> {
    let (op_sig, _reg_sig) = op_signatures(program);
    let mut out = Vec::new();
    for (i, op) in program.ops().iter().enumerate() {
        if let Op::Ugen { buffer_len, .. } = *op
            && buffer_len > 0
        {
            out.push(BufferSlot { sig: op_sig[i], len: buffer_len });
        }
    }
    out
}

/// Whether the new program may donate the old program's buffer arena whole (an O(1) `Vec`
/// swap). Two conditions must both hold:
///
/// 1. the layouts are identical (`old == new`), so the arenas are the same shape; and
/// 2. every buffer region is *uniquely* identified by its `(signature, len)`.
///
/// Condition 2 is the subtle one. Donation swaps the flat arena by raw offset, but two
/// structurally-identical buffers (e.g. two `record`/`play` loops recording the same kind of
/// source) are indistinguishable after compile — so if the user reorders or renames them, the
/// op-order layout still matches yet each loop's offset has moved, and the swap would land each
/// loop's audio under the wrong name. When any `(signature, len)` repeats we cannot prove the
/// offsets line up, so we decline (the buffers reset to silence) rather than risk cross-swapping
/// the wrong contents. The common cases — one delay, one looper, a delay plus a looper — are all
/// distinct and donate as before.
pub fn can_donate(old: &[BufferSlot], new: &[BufferSlot]) -> bool {
    old == new && all_distinct(new)
}

/// Whether every entry is unique. `n` is the buffer count of one patch (tiny), so the O(n²)
/// scan is cheaper than hashing and allocation-free.
fn all_distinct(slots: &[BufferSlot]) -> bool {
    slots.iter().enumerate().all(|(i, a)| !slots[..i].contains(a))
}

/// One buffer region of a program: the structural signature of the op that owns it, its base
/// offset in the flat buffer arena, and its length — in op order, **deduped by base** (a named
/// buffer shared by `record`+`play` is one region, identified by its first op in op order).
/// Where [`BufferSlot`] decides whole-arena *donation*, regions are the unit of the per-buffer
/// *copy* plan used by faded swaps, where the displaced engine keeps using its own arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferRegion {
    pub sig: u64,
    pub base: u32,
    pub len: u32,
}

/// The buffer regions of `program`, in op order, deduped by base offset.
pub fn buffer_regions(program: &Program) -> Vec<BufferRegion> {
    let (op_sig, _reg_sig) = op_signatures(program);
    let mut out: Vec<BufferRegion> = Vec::new();
    for (i, op) in program.ops().iter().enumerate() {
        if let Op::Ugen { buffer_base, buffer_len, .. } = *op
            && buffer_len > 0
            && !out.iter().any(|r| r.base == buffer_base)
        {
            out.push(BufferRegion { sig: op_sig[i], base: buffer_base, len: buffer_len });
        }
    }
    out
}

/// One buffer copy: `len` f32s from the old arena's `old_base` to the new arena's `new_base`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufCopy {
    pub old_base: u32,
    pub new_base: u32,
    pub len: u32,
}

/// A bounded per-buffer copy plan for a faded swap, plus how many matched regions were
/// declined for exceeding the budget (they reset to silence through the fade; the frontends
/// surface the count as a diagnostic).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BufCopyPlan {
    pub copies: Vec<BufCopy>,
    pub declined: u32,
}

/// Per-buffer copy cap, in samples (f32s): 2^18 = 1 MiB ≈ 5.46 s mono @ 48 kHz. A 1 s stereo
/// delay (384 KB) fits comfortably; minutes-long sampler material declines alone.
pub const MAX_COPY_PER_BUFFER: u32 = 1 << 18;

/// Total per-adoption copy cap, in samples: 2^20 = 4 MiB ≈ 0.13–0.4 ms of memcpy — a small
/// slice of the ≈ 2.67 ms callback budget at 48 kHz / 128 frames, applied on the audio thread.
pub const MAX_COPY_TOTAL: u32 = 1 << 20;

/// Match the new program's buffer regions to the old's by `((signature, len), ordinal)` —
/// the same keying as [`plan`] — and emit the copies that fit the budget. Built on the
/// control thread, so an over-budget decline surfaces immediately as a diagnostic, and the
/// audio thread only ever applies a pre-bounded list.
///
/// Copies (not donation) are what a *faded* swap needs: the outgoing engine is still audibly
/// using — and writing — its own arena for the duration of the fade, so the arena cannot move.
/// After the copy both engines write identical samples into their respective arenas, which is
/// what keeps the crossfade cancelling on unchanged delay lines.
pub fn buffer_copy_plan(old: &[BufferRegion], new: &[BufferRegion]) -> BufCopyPlan {
    use std::collections::HashMap;
    let mut index: HashMap<((u64, u32), u32), u32> = HashMap::new();
    let mut seen: HashMap<(u64, u32), u32> = HashMap::new();
    for r in old {
        let ord = seen.entry((r.sig, r.len)).or_insert(0);
        index.insert(((r.sig, r.len), *ord), r.base);
        *ord += 1;
    }
    let mut taken: HashMap<(u64, u32), u32> = HashMap::new();
    let mut plan = BufCopyPlan::default();
    let mut total: u32 = 0;
    for r in new {
        let ord = taken.entry((r.sig, r.len)).or_insert(0);
        if let Some(&old_base) = index.get(&((r.sig, r.len), *ord)) {
            if r.len > MAX_COPY_PER_BUFFER || total + r.len > MAX_COPY_TOTAL {
                plan.declined += 1;
            } else {
                total += r.len;
                plan.copies.push(BufCopy { old_base, new_base: r.base, len: r.len });
            }
        }
        *ord += 1;
    }
    plan
}

/// Match the new program's stateful ops to the old's by `(signature, ordinal)` and emit
/// the state copies for every pair whose slot counts agree.
pub fn plan(old: &[StatefulNode], new: &[StatefulNode]) -> Vec<Migrate> {
    use std::collections::HashMap;
    // Index the old nodes by (signature, ordinal-within-signature).
    let mut index: HashMap<(u64, u32), (u32, u32)> = HashMap::new();
    let mut seen: HashMap<u64, u32> = HashMap::new();
    for n in old {
        let ord = seen.entry(n.sig).or_insert(0);
        index.insert((n.sig, *ord), (n.base, n.slots));
        *ord += 1;
    }
    let mut taken: HashMap<u64, u32> = HashMap::new();
    let mut migrations = Vec::new();
    for n in new {
        let ord = taken.entry(n.sig).or_insert(0);
        if let Some(&(old_base, slots)) = index.get(&(n.sig, *ord))
            && slots == n.slots
        {
            // The new node's plane is authoritative: a shared signature implies the same
            // kind, so old and new live in the same plane.
            migrations.push(Migrate {
                plane: n.plane,
                old_base,
                new_base: n.base,
                slots: n.slots,
            });
        }
        *ord += 1;
    }
    migrations
}

// A small deterministic FNV-1a fold. `DefaultHasher` is randomized per process, which
// would still be consistent within one run (old and new sigs are computed together) —
// but an explicit fold makes the signature reproducible and obviously order-sensitive.
const CONST_TAG: u64 = 0x01;
const UGEN_TAG: u64 = 0x02;
const FBREAD_TAG: u64 = 0x03;
const FB_TAG: u64 = 0x04;
const INPUT_TAG: u64 = 0x05;
const OUTPUT_TAG: u64 = 0x06;
const CONTROL_TAG: u64 = 0x07;
const NOW_TAG: u64 = 0x08;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn hash(tag: u64, parts: &[u64]) -> u64 {
    let mut h = mix(FNV_OFFSET, tag);
    for &p in parts {
        h = mix(h, p);
    }
    h
}

fn mix(mut h: u64, word: u64) -> u64 {
    for b in word.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::compile;
    use crate::graph::Graph;
    use crate::testutil::{graph_of, osc, osc_gain, u};

    const SR: f32 = 48_000.0;

    fn sigs(graph: &Graph) -> Vec<StatefulNode> {
        signatures(&compile(graph, SR))
    }

    fn bufs(graph: &Graph) -> Vec<BufferSlot> {
        buffer_layout(&compile(graph, SR))
    }

    // ---- Fixture graphs (the Forth programs the reconcile tests used to parse) ----

    /// `noise record X drop play X out` — one looper: record sinks noise into a named buffer,
    /// play reads it back.
    fn looper_noise() -> Graph {
        graph_of(|g| {
            let b = g.new_buffer(1.0);
            let n = g.ugen(u("noise"), vec![]);
            let r = g.ugen_buf(u("record"), vec![n], b);
            g.add_sink(r);
            vec![g.ugen_buf(u("play"), vec![], b)]
        })
    }

    /// `noise record a drop play a  noise record b drop play b + out` — two independent loopers.
    fn two_loopers() -> Graph {
        graph_of(|g| {
            let looper = |g: &mut Graph| {
                let b = g.new_buffer(1.0);
                let n = g.ugen(u("noise"), vec![]);
                let r = g.ugen_buf(u("record"), vec![n], b);
                g.add_sink(r);
                g.ugen_buf(u("play"), vec![], b)
            };
            let a = looper(g);
            let b = looper(g);
            vec![g.ugen(u("+"), vec![a, b])]
        })
    }

    /// `freq sine cutoff lpf out`.
    fn sine_lpf(freq: f32, cutoff: f32) -> Graph {
        graph_of(|g| {
            let f = g.constant(freq);
            let s = g.ugen(u("sine"), vec![f]);
            let c = g.constant(cutoff);
            vec![g.ugen(u("lpf"), vec![s, c])]
        })
    }

    /// `freq sine sine out` — a sine fed by another sine.
    fn nested_sine(freq: f32) -> Graph {
        graph_of(|g| {
            let f = g.constant(freq);
            let inner = g.ugen(u("sine"), vec![f]);
            vec![g.ugen(u("sine"), vec![inner])]
        })
    }

    /// `f1 sine f2 sine + out` — two sines summed.
    fn two_sines(f1: f32, f2: f32) -> Graph {
        graph_of(|g| {
            let c1 = g.constant(f1);
            let s1 = g.ugen(u("sine"), vec![c1]);
            let c2 = g.constant(f2);
            let s2 = g.ugen(u("sine"), vec![c2]);
            vec![g.ugen(u("+"), vec![s1, s2])]
        })
    }

    /// `1 0.05 delay 0.01 delay out` — two anonymous delay lines in series.
    fn two_delays() -> Graph {
        graph_of(|g| {
            let s = g.constant(1.0);
            let t1 = g.constant(0.05);
            let d1 = g.ugen(u("delay"), vec![s, t1]);
            let t2 = g.constant(0.01);
            vec![g.ugen(u("delay"), vec![d1, t2])]
        })
    }

    /// `buf big 12  noise record big drop  play big  0.01 delay out` — a 12 s named buffer
    /// (oversized for the copy cap) feeding a small delay.
    fn big_looper() -> Graph {
        graph_of(|g| {
            let big = g.new_buffer(12.0);
            let n = g.ugen(u("noise"), vec![]);
            let r = g.ugen_buf(u("record"), vec![n], big);
            g.add_sink(r);
            let p = g.ugen_buf(u("play"), vec![], big);
            let t = g.constant(0.01);
            vec![g.ugen(u("delay"), vec![p, t])]
        })
    }

    /// `buf a 5 … buf e 5  noise record a drop … noise record e drop  0 out` — five 5 s named
    /// buffers, each recorded from its own noise; the output is a bare constant.
    fn five_buffers() -> Graph {
        graph_of(|g| {
            for _ in 0..5 {
                let b = g.new_buffer(5.0);
                let n = g.ugen(u("noise"), vec![]);
                let r = g.ugen_buf(u("record"), vec![n], b);
                g.add_sink(r);
            }
            vec![g.constant(0.0)]
        })
    }

    /// `b' 100 lpf as b out` — a one-pole in a single feedback loop.
    fn fb_lpf() -> Graph {
        graph_of(|g| {
            let b = g.new_bus();
            let read = g.fb_read(b);
            let c = g.constant(100.0);
            let lpf = g.ugen(u("lpf"), vec![read, c]);
            g.set_bus_source(b, lpf);
            vec![lpf]
        })
    }

    /// `a' drop b' 100 lpf as b out 0 as a` — the same loop plus an unrelated bus `a` declared
    /// first (it shifts `b`'s slot). Bus `a` is read (dropped) and fed a constant.
    fn fb_lpf_two_bus() -> Graph {
        graph_of(|g| {
            let bus_a = g.new_bus(); // `a'` allocates bus a first
            let _read_a = g.fb_read(bus_a); // `a'` then `drop` — the read is discarded
            let bus_b = g.new_bus();
            let read_b = g.fb_read(bus_b);
            let c = g.constant(100.0);
            let lpf = g.ugen(u("lpf"), vec![read_b, c]);
            g.set_bus_source(bus_b, lpf); // `as b`
            let zero = g.constant(0.0);
            g.set_bus_source(bus_a, zero); // `0 as a`
            vec![lpf]
        })
    }

    /// `fb e 2  [ 1 2 ] e' gain * +  as e  e' out` — a width-2 feedback latch: each channel
    /// sums a constant with `gain ×` its own previous sample, then the latch is read out.
    fn stereo_latch(gain: f32) -> Graph {
        graph_of(|g| {
            let (b0, b1) = (g.new_bus(), g.new_bus());
            let (c1, c2) = (g.constant(1.0), g.constant(2.0));
            let (er0, er1) = (g.fb_read(b0), g.fb_read(b1));
            let k = g.constant(gain);
            let m0 = g.ugen(u("*"), vec![er0, k]);
            let m1 = g.ugen(u("*"), vec![er1, k]);
            let a0 = g.ugen(u("+"), vec![c1, m0]);
            let a1 = g.ugen(u("+"), vec![c2, m1]);
            g.set_bus_source(b0, a0);
            g.set_bus_source(b1, a1);
            vec![g.fb_read(b0), g.fb_read(b1)]
        })
    }

    #[test]
    fn donation_declines_when_buffers_are_indistinguishable() {
        // A single looper: `record` and `play` have distinct signatures, so its arena is
        // unambiguous and donates (a re-eval keeps the loop, as before).
        let one = bufs(&looper_noise());
        assert!(can_donate(&one, &one), "a single looper's arena is unambiguous");

        // Two structurally-identical loops compile to identical `(sig, len)` regions — after
        // compile there is no way to tell which is which, so a reorder could silently cross-swap
        // their recorded audio. Donation must decline rather than risk the wrong contents.
        let two = bufs(&two_loopers());
        assert!(
            !can_donate(&two, &two),
            "indistinguishable loops must not donate (would cross-swap on reorder)"
        );
    }

    #[test]
    fn finds_stateful_ops_with_their_state_layout() {
        let one = sigs(&osc("sine", 440.0));
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].slots, 1);
        assert_eq!(one[0].base, 0);

        // sine (1 slot @ 0) then lpf (1 slot @ 1); the `+`/`*`/consts carry no state.
        let two = sigs(&sine_lpf(440.0, 800.0));
        assert_eq!(two.len(), 2);
        assert_eq!(two[0].base, 0);
        assert_eq!(two[1].base, 1);
    }

    #[test]
    fn signature_ignores_constant_values() {
        assert_eq!(sigs(&osc("sine", 440.0))[0].sig, sigs(&osc("sine", 441.0))[0].sig);
    }

    #[test]
    fn signature_distinguishes_ugen_kind() {
        assert_ne!(sigs(&osc("sine", 440.0))[0].sig, sigs(&osc("saw", 440.0))[0].sig);
    }

    #[test]
    fn signature_distinguishes_upstream_structure() {
        // A sine fed by a constant vs a sine fed by another oscillator: different
        // identity. In `440 sine sine` the inner sine ([0]) is fed by the constant, so it
        // matches the plain `440 sine`; the outer sine ([1]) is fed by a sine, so differs.
        let plain = sigs(&osc("sine", 440.0))[0].sig;
        let nested = sigs(&nested_sine(440.0));
        assert_eq!(nested[0].sig, plain, "inner sine is fed by the constant");
        assert_ne!(nested[1].sig, plain, "outer sine is fed by a sine");
    }

    #[test]
    fn plan_migrates_across_a_parameter_tweak() {
        let p = plan(&sigs(&osc_gain("sine", 440.0, 0.2)), &sigs(&osc_gain("sine", 440.0, 0.3)));
        assert_eq!(p, vec![Migrate { plane: Plane::State, old_base: 0, new_base: 0, slots: 1 }]);
    }

    #[test]
    fn plan_resets_on_kind_change() {
        assert_eq!(plan(&sigs(&osc("sine", 440.0)), &sigs(&osc("saw", 440.0))), vec![]);
    }

    #[test]
    fn plan_matches_siblings_by_ordinal() {
        let p = plan(
            &sigs(&two_sines(440.0, 660.0)),
            &sigs(&two_sines(441.0, 661.0)),
        );
        assert_eq!(
            p,
            vec![
                Migrate { plane: Plane::State, old_base: 0, new_base: 0, slots: 1 },
                Migrate { plane: Plane::State, old_base: 1, new_base: 1, slots: 1 },
            ]
        );
    }

    #[test]
    fn plan_is_empty_from_a_stateless_predecessor() {
        assert_eq!(plan(&[], &sigs(&osc("sine", 440.0))), vec![]);
    }

    fn regions(graph: &Graph) -> Vec<BufferRegion> {
        buffer_regions(&compile(graph, SR))
    }

    #[test]
    fn buffer_regions_carry_bases_in_op_order() {
        // Two anonymous delays: two regions, laid out back to back in the flat arena.
        let r = regions(&two_delays());
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].base, 0);
        assert_eq!(r[1].base, r[0].len, "the second region starts where the first ends");
    }

    #[test]
    fn buffer_regions_dedupe_a_shared_named_buffer() {
        // `record l` and `play l` address ONE named buffer — one region, not two; a copy
        // plan that listed it twice would memcpy it twice.
        let r = regions(&looper_noise());
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn copy_plan_matches_by_sig_len_and_ordinal() {
        let r = regions(&two_delays());
        let p = buffer_copy_plan(&r, &r);
        assert_eq!(p.declined, 0);
        assert_eq!(p.copies.len(), 2, "a self-edit copies every region");
        for c in &p.copies {
            assert_eq!(c.old_base, c.new_base, "identical layouts map region onto itself");
        }
    }

    #[test]
    fn copy_plan_declines_an_oversized_buffer_and_counts_it() {
        // 12 s @ 48 kHz = 576_000 samples > MAX_COPY_PER_BUFFER (262_144): declined alone;
        // the small delay alongside it still copies.
        let r = regions(&big_looper());
        let p = buffer_copy_plan(&r, &r);
        assert_eq!(p.declined, 1, "the oversized buffer declines");
        assert_eq!(p.copies.len(), 1, "the small delay still copies");
    }

    #[test]
    fn copy_plan_enforces_the_total_budget() {
        // Five 5 s named buffers: 240_000 samples each (under the per-buffer cap); four fit
        // the 1_048_576-sample total, the fifth would exceed it and declines.
        let r = regions(&five_buffers());
        assert_eq!(r.len(), 5);
        let p = buffer_copy_plan(&r, &r);
        assert_eq!(p.copies.len(), 4, "four regions fit the total budget");
        assert_eq!(p.declined, 1, "the fifth declines on the total cap");
    }

    #[test]
    fn feedback_read_signature_is_stable_under_bus_renumbering() {
        // Editing from one feedback bus to two — an unrelated bus `a` declared first
        // shifts `b`'s slot — must not reset the filter that reads `b`. Its structural
        // signature (and the bus's, since the bus is fed by it) must not depend on the
        // slot index, or both migrations are lost across the hot-swap: an audible click.
        let before = sigs(&fb_lpf());
        let after = sigs(&fb_lpf_two_bus());
        assert_eq!(plan(&before, &after).len(), 2, "lpf and bus `b` should both migrate");
    }

    #[test]
    fn a_multichannel_latch_migrates_every_channel() {
        // A stereo feedback latch carries both of its bus slots across a parameter tweak — each
        // channel's bus is keyed by its own source structure, so a const change migrates both.
        let before = sigs(&stereo_latch(0.5));
        let after = sigs(&stereo_latch(0.6));
        let buses = plan(&before, &after).into_iter().filter(|m| m.plane == Plane::Bus).count();
        assert_eq!(buses, 2, "both feedback channels carry across the edit");
    }
}
