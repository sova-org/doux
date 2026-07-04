//! Lower a [`Graph`] into a [`Program`].
//!
//! A depth-first post-order walk from the output node emits each reachable node
//! exactly once. Emitting a node's inputs before the node itself guarantees the
//! topological invariant (inputs get lower register indices). Memoising the
//! register of each visited node turns shared nodes (a DAG) into a single op.
//!
//! The walk is iterative (an explicit `work` stack in [`Lowering::emit`]), so an
//! arbitrarily deep graph lowers without overflowing the call stack.

use crate::graph::{Graph, Node, NodeId};
use crate::ir::{Feedback, Op, Program, Reg};
use crate::ugen;

pub fn compile(graph: &Graph, sample_rate: f32) -> Program {
    // Each feedback bus owns one slot in the bus plane (slot = bus index), a separate arena
    // from per-UGen state — so the number of buses never shifts a UGen's `state_base`.
    let n_buses = graph.bus_sources().len() as u32;

    // Lay out each named buffer once, at the front of the buffer arena, sizing it from seconds
    // at this program's sample rate; every op that names it shares the region. Anonymous
    // per-instance buffers (e.g. `delay`) are appended after, during the walk.
    let mut buf_bases = Vec::with_capacity(graph.buffers().len());
    let mut buf_lens = Vec::with_capacity(graph.buffers().len());
    let mut named_total = 0u32;
    for &seconds in graph.buffers() {
        let len = buffer_samples(seconds, sample_rate);
        buf_bases.push(named_total);
        buf_lens.push(len);
        named_total += len;
    }

    let mut ctx = Lowering {
        graph,
        ops: Vec::new(),
        inputs: Vec::new(),
        reg_of: vec![None; graph.len()],
        reg_cursor: 0,
        state_len: 0,
        buffer_len: named_total,
        buf_bases,
        buf_lens,
        in_channels: 0,
        control_len: 0,
        initial_state: Vec::new(),
        seed_ordinal: 0,
    };

    let mut outputs: Vec<Reg> = graph
        .outputs()
        .iter()
        .map(|&node| ctx.emit(node))
        .collect();

    // Observation taps (`scope name`) are extra outputs appended after the audio channels: the
    // VM/JIT already write every `outputs` register, so a tap rides out for free, and the host
    // meters the trailing ones (peak+rms) instead of routing them to the device. A scoped node
    // shared with an audio output is memoised (no duplicate op); a scoped mid-graph node that is
    // otherwise unreachable becomes live so its level can be observed.
    let audio_channels = outputs.len();
    let mut tap_names = Vec::with_capacity(graph.taps().len());
    for (name, node) in graph.taps() {
        let reg = ctx.emit(*node);
        outputs.push(reg);
        tap_names.push(name.clone());
    }

    // Each written bus is also a root: emit its source so the write-back has a register,
    // and record the store into the bus's slot (the read side is an `Op::FbRead`).
    let mut feedbacks = Vec::new();
    for (bus, source) in graph.bus_sources().iter().enumerate() {
        if let Some(&node) = source.as_ref() {
            let reg = ctx.emit(node);
            feedbacks.push(Feedback { slot: bus as u32, source: reg });
        }
    }

    // Sinks (e.g. `record`) write a buffer as a side effect, so they must run even when their
    // passthrough output is unused — emit them as roots, after the outputs they may follow.
    for &node in graph.sinks() {
        ctx.emit(node);
    }

    Program {
        ops: ctx.ops,
        register_count: ctx.reg_cursor as usize,
        inputs: ctx.inputs,
        state_len: ctx.state_len as usize,
        initial_state: ctx.initial_state,
        bus_len: n_buses as usize,
        buffer_len: ctx.buffer_len as usize,
        in_channels: ctx.in_channels as usize,
        control_len: ctx.control_len as usize,
        voice_count: graph.voice_count(),
        outputs,
        audio_channels,
        tap_names,
        params: graph.params().to_vec(),
        feedbacks,
        sample_rate,
    }
}

struct Lowering<'g> {
    graph: &'g Graph,
    ops: Vec<Op>,
    inputs: Vec<Reg>,
    reg_of: Vec<Option<Reg>>,
    /// Next free register. Advances by each op's output count (one for a leaf, the UGen's
    /// `outputs` for a generator), so an op's first output register is the cursor before it
    /// is emitted — and inputs, emitted earlier, always hold lower indices.
    reg_cursor: u32,
    state_len: u32,
    buffer_len: u32,
    /// Base offset (f32s) of each named buffer's shared region, indexed by `BufId`.
    buf_bases: Vec<u32>,
    /// Sample length of each named buffer, indexed by `BufId`.
    buf_lens: Vec<u32>,
    in_channels: u32,
    control_len: u32,
    /// Sparse fresh-init for the state plane: `(absolute slot, value)`. Only noise sources push
    /// here — their counter slot gets a per-instance seed so co-existing instances decorrelate.
    initial_state: Vec<(u32, f32)>,
    /// Running index of seeded ops, feeding [`ugen::noise_seed`] so each gets a distinct seed.
    seed_ordinal: u32,
}

/// Samples for a buffer of `seconds` at `sample_rate`: at least one, and capped so a typo
/// cannot request a pathological allocation. Non-power-of-two in general (sized to the request).
fn buffer_samples(seconds: f32, sample_rate: f32) -> u32 {
    const MAX: u32 = 1 << 24; // ~349 s at 48 kHz (~67 MB) — generous but bounded
    let samples = (seconds.max(0.0) * sample_rate).ceil();
    (samples as u32).clamp(1, MAX)
}

impl Lowering<'_> {
    /// Lower the cone under `root` and return the register holding its value. Iterative
    /// post-order DFS — a recursive walk would overflow the stack on a pathological chain
    /// (thousands of nodes deep), so depth is bounded by the heap `work` stack instead. Each
    /// node is *entered* (scheduling its children, then itself for *exit*) and later *exited*
    /// (emitting its op once its children hold registers). `reg_of` memoises, so a shared node
    /// is emitted exactly once. The emission order is identical to the old recursive walk —
    /// children left-to-right, then the node — which the bit-exact harness pins.
    fn emit(&mut self, root: NodeId) -> Reg {
        enum Step {
            Enter(NodeId),
            Exit(NodeId),
        }
        let mut work = vec![Step::Enter(root)];
        while let Some(step) = work.pop() {
            match step {
                Step::Enter(id) => {
                    if self.reg_of[id.0 as usize].is_some() {
                        continue; // already emitted (shared node)
                    }
                    // Schedule this node's Exit, then its children above it so they emit first.
                    // Children are pushed in reverse so they pop left-to-right (matching the
                    // recursive order). Leaves have no children — they just await their Exit.
                    match self.graph.node(id) {
                        Node::Output { source, .. } => {
                            work.push(Step::Exit(id));
                            work.push(Step::Enter(source));
                        }
                        Node::Ugen { inputs, .. } => {
                            work.push(Step::Exit(id));
                            for &input in inputs.iter().rev() {
                                work.push(Step::Enter(input));
                            }
                        }
                        Node::Const(_)
                        | Node::FbRead { .. }
                        | Node::Input { .. }
                        | Node::Control { .. }
                        | Node::Now => work.push(Step::Exit(id)),
                    }
                }
                Step::Exit(id) => {
                    if self.reg_of[id.0 as usize].is_some() {
                        continue; // emitted between this Exit being scheduled and reached
                    }
                    // Resolve the node to the register holding its value. Most nodes emit one op
                    // (a block of output registers); `Output` emits none — it just names a port
                    // of its source, so a multi-output generator tapped several ways stays one op.
                    let reg = match self.graph.node(id) {
                        Node::Output { source, port } => {
                            let src = self.reg_of[source.0 as usize].expect("source emitted first");
                            Reg(src.0 + port)
                        }
                        Node::Const(v) => self.push_op(Op::Const(v)),
                        // A leaf reading the bus's pre-assigned slot in the bus plane.
                        Node::FbRead { bus } => self.push_op(Op::FbRead { slot: bus.0 }),
                        // A leaf reading an input channel; widens the declared input count.
                        Node::Input { channel } => {
                            self.in_channels = self.in_channels.max(channel + 1);
                            self.push_op(Op::Input { channel })
                        }
                        // A leaf reading a control lane; widens the declared control width.
                        Node::Control { lane } => {
                            self.control_len = self.control_len.max(lane + 1);
                            self.push_op(Op::Control { lane })
                        }
                        // A leaf reading the executor's sample clock; no host plane to widen.
                        Node::Now => self.push_op(Op::Now),
                        Node::Ugen { ugen, inputs, buffer } => {
                            // Inputs were emitted first (entered before this Exit), so their
                            // registers exist; place them contiguously in the arena.
                            let regs: Vec<Reg> = inputs
                                .iter()
                                .map(|&input| self.reg_of[input.0 as usize].expect("input emitted first"))
                                .collect();
                            let input_count = regs.len() as u32;
                            let input_start = self.inputs.len() as u32;
                            self.inputs.extend_from_slice(&regs);
                            let state_base = self.state_len;
                            self.state_len += ugen::def(ugen).state_slots as u32;
                            // Seed a noise source's counter slot so co-existing instances
                            // decorrelate (without this each starts at counter 0 and emits the
                            // identical stream). `seed_ordinal` is a running index over seeded
                            // ops; reconcile carries a matched op's live counter over this seed.
                            if let Some(k) = ugen::seed_slot(ugen::def(ugen).name) {
                                self.initial_state
                                    .push((state_base + k as u32, ugen::noise_seed(self.seed_ordinal)));
                                self.seed_ordinal += 1;
                            }
                            // A named buffer shares its pre-laid region; otherwise the generator
                            // gets a fresh anonymous region of its row's `buffer_len`.
                            let (buffer_base, buffer_len) = match buffer {
                                Some(buf) => (self.buf_bases[buf.0 as usize], self.buf_lens[buf.0 as usize]),
                                None => {
                                    let len = ugen::def(ugen).buffer_len as u32;
                                    let base = self.buffer_len;
                                    self.buffer_len += len;
                                    (base, len)
                                }
                            };
                            self.push_op(Op::Ugen {
                                ugen, input_start, input_count, state_base, buffer_base, buffer_len,
                            })
                        }
                    };
                    self.reg_of[id.0 as usize] = Some(reg);
                }
            }
        }
        self.reg_of[root.0 as usize].expect("root emitted")
    }

    /// Append `op`, returning the first register it writes; advances the cursor past its
    /// output block (one register for a leaf, the UGen's `outputs` for a generator).
    fn push_op(&mut self, op: Op) -> Reg {
        let reg = Reg(self.reg_cursor);
        let outputs = match &op {
            Op::Ugen { ugen, .. } => ugen::def(*ugen).outputs as u32,
            _ => 1,
        };
        self.ops.push(op);
        self.reg_cursor += outputs;
        reg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{graph_of, osc_gain, u};

    #[test]
    fn dup_shares_a_single_node() {
        // `dup` reuses the NodeId, so the sine must be emitted once:
        // Const(440), Sine, Add  => 3 ops.
        let graph = graph_of(|g| {
            let f = g.constant(440.0);
            let s = g.ugen(u("sine"), vec![f]);
            vec![g.ugen(u("+"), vec![s, s])] // `dup`: both `+` inputs are the same node
        });
        let program = compile(&graph, 48_000.0);
        assert_eq!(program.num_registers(), 3);
        assert_eq!(program.state_len(), 1); // one phase, shared
    }

    #[test]
    fn topological_order_inputs_before_consumers() {
        let graph = osc_gain("sine", 440.0, 0.5);
        let program = compile(&graph, 48_000.0);
        // Output is the last op (the multiply).
        let out = program.outputs()[0];
        assert_eq!(out.0 as usize, program.num_registers() - 1);
    }

    #[test]
    fn scope_appends_a_tap_output_after_the_audio_channels() {
        // `scope osc` adds a trailing tap output; the one audio channel is unchanged, and the
        // tap name is recorded aligned with `outputs()[audio_channels()..]`.
        let g = graph_of(|g| {
            let f = g.constant(440.0);
            let s = g.ugen(u("sine"), vec![f]);
            g.add_tap("osc".to_string(), s); // `scope osc` taps the sine, leaving it in the chain
            let a = g.constant(0.3);
            vec![g.ugen(u("*"), vec![s, a])]
        });
        let p = compile(&g, 48_000.0);
        assert_eq!(p.audio_channels(), 1, "one audio channel");
        assert_eq!(p.outputs().len(), 2, "audio channel + one tap");
        assert_eq!(p.tap_names(), &["osc".to_string()]);
    }

    #[test]
    fn a_program_without_scope_has_no_taps() {
        let g = osc_gain("sine", 440.0, 0.3);
        let p = compile(&g, 48_000.0);
        assert_eq!(p.audio_channels(), p.outputs().len());
        assert!(p.tap_names().is_empty());
    }

    #[test]
    fn a_very_deep_graph_compiles_without_overflowing_the_stack() {
        // A pathologically deep linear chain (a generated/pasted/macro-expanded patch) must
        // lower without a recursive stack overflow — the walk is iterative, bounded by heap.
        let sine = crate::ugen::lookup("sine").expect("sine exists");
        let mut g = Graph::new();
        let mut node = g.constant(440.0);
        const N: usize = 200_000;
        for _ in 0..N {
            node = g.ugen(sine, vec![node]);
        }
        g.set_outputs(vec![node]);
        let program = compile(&g, 48_000.0);
        assert_eq!(program.num_registers(), N + 1, "const + N sines = N+1 registers");
    }
}
