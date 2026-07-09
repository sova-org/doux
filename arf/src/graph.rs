//! The authoring representation: a directed acyclic graph of audio nodes.
//!
//! The Forth front-end builds a `Graph` bottom-up. Each node produces one signal
//! value per sample. Sharing (a node feeding more than one consumer) is expressed
//! simply by reusing a [`NodeId`] — the compiler emits each shared node once.

use crate::ugen::UGenId;
use serde::{Deserialize, Serialize};

/// The widest a channel-list may grow. Bounds the realtime scratch and catches
/// runaway multichannel expansion at parse time.
pub const MAX_CHANNELS: usize = 64;

/// Lane of the gate signal (1.0 held, 0.0 released).
pub const GATE_LANE: usize = 0;
/// Lane of the note frequency (Hz of the current note).
pub const NOTEFREQ_LANE: usize = 1;
/// Lane of the note velocity (0..1 of the last note-on).
pub const VEL_LANE: usize = 2;
/// The transport's beats-per-second lane, just past the note lanes. The host latches the
/// tempo here each block (control-rate); `bps` reads it. Seeded with the host's default
/// tempo so a patch never reads 0 before the first block.
pub const BPS_LANE: usize = 3;
/// First lane of the named-parameter block (`param name default`), just past the transport
/// lane. A declaration's lane is `PARAM_BASE + declaration index`.
pub const PARAM_BASE: usize = BPS_LANE + 1;
/// How many named parameters one patch may declare. Bounds the control plane so the host can
/// keep a fixed-size per-instance control block.
pub const MAX_PARAMS: usize = 16;
/// Total control-plane width: the note lanes, the transport lane, and the named-parameter
/// block. The host allocates a control block this wide; a program reads only its prefix.
pub const CONTROL_WIDTH: usize = PARAM_BASE + MAX_PARAMS;

/// A handle to a node, i.e. its index into [`Graph::nodes`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct NodeId(pub(crate) u32);

/// A handle to a feedback bus, i.e. its index into [`Graph::bus_sources`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BusId(pub(crate) u32);

/// A handle to a named buffer, i.e. its index into [`Graph::buffers`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BufId(pub(crate) u32);

/// A node in the audio graph. `Const` is the only leaf carrying a value; every other
/// node is a UGen invocation referencing its inputs by [`NodeId`]. The kind and the
/// meaning of each input live in the [`UGenId`]'s row of the contract table.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
    /// A constant signal.
    Const(f32),
    /// A unit generator applied to its inputs (in source order). `buffer` names the shared
    /// sample-memory region it reads/writes, if any (`play`/`record`); `None` means the
    /// generator uses its own anonymous per-instance buffer (`delay`) or no buffer at all.
    Ugen {
        ugen: UGenId,
        inputs: Vec<NodeId>,
        buffer: Option<BufId>,
    },
    /// A read of feedback bus `bus`, delayed one sample. It is a leaf: its value is the
    /// bus's previous-sample state, so it carries no graph input and breaks the cycle.
    FbRead { bus: BusId },
    /// A read of audio input `channel` (the current frame's sample). A leaf: the value
    /// comes from the host's input block, not from another node.
    Input { channel: u32 },
    /// A read of control `lane` (a host-supplied value: the note's gate/notefreq/vel, the
    /// transport tempo, or a named parameter). A leaf, like [`Node::Input`], but the value comes
    /// from the host's per-block control plane, held constant for the whole block (control-rate).
    Control { lane: u32 },
    /// A read of the global sample clock as a signal. A leaf, like [`Node::Input`], but supplied
    /// by the executor: the current frame's absolute position windowed into `NOW_WINDOW`. The
    /// foundation of musical time — envelopes and oscillators read it (see [`crate::ir::Op::Now`]).
    Now,
    /// A selection of one output `port` of multi-output `source` (a UGen node). A zero-cost
    /// proxy (SuperCollider's OutputProxy): it compiles to no op, just naming the register the
    /// `source` already produces — so a generator tapped several ways stays one shared instance.
    Output { source: NodeId, port: u32 },
}

/// An audio graph under construction, plus the nodes designated as its output channels
/// (sinks). A mono graph has one output; a multichannel graph has several.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Graph {
    nodes: Vec<Node>,
    outputs: Vec<NodeId>,
    /// The source node feeding each feedback bus, indexed by [`BusId`]. `None` until the
    /// parser resolves the `as` binding a `name'` latch reads (an unbound latch is a
    /// parse error, so a compiled program never carries a sourceless bus).
    bus_sources: Vec<Option<NodeId>>,
    /// Declared named buffers: length in seconds, indexed by [`BufId`]. The compiler resolves
    /// each to a sample count at the program's sample rate and lays one region per buffer,
    /// shared by every op (e.g. a `record`/`play` pair) that names it.
    buffers: Vec<f32>,
    /// Nodes whose side effect (a buffer write by `record`) must run even when their value is
    /// unused — so the compiler keeps them as roots, like a written feedback bus.
    sinks: Vec<NodeId>,
    /// Declared named parameters (`param name default`), in declaration order. Entry `i` owns
    /// lane `PARAM_BASE + i`; the default holds until the host writes the lane. `default` so
    /// graphs serialized before the field existed still load.
    #[serde(default)]
    params: Vec<(String, f32)>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        id
    }

    pub fn constant(&mut self, value: f32) -> NodeId {
        self.push(Node::Const(value))
    }

    /// Apply a unit generator to `inputs` (in source order). The arity is the caller's
    /// responsibility — the Forth front-end reads it from the contract table.
    pub fn ugen(&mut self, ugen: UGenId, inputs: Vec<NodeId>) -> NodeId {
        self.push(Node::Ugen {
            ugen,
            inputs,
            buffer: None,
        })
    }

    /// Apply a unit generator that reads/writes a named `buffer` (e.g. `play`/`record`).
    pub fn ugen_buf(&mut self, ugen: UGenId, inputs: Vec<NodeId>, buffer: BufId) -> NodeId {
        self.push(Node::Ugen {
            ugen,
            inputs,
            buffer: Some(buffer),
        })
    }

    /// Declare a named buffer of `seconds`, returning its handle. The compiler sizes it at the
    /// program's sample rate; several ops may share one buffer by naming it.
    pub fn new_buffer(&mut self, seconds: f32) -> BufId {
        let id = BufId(self.buffers.len() as u32);
        self.buffers.push(seconds);
        id
    }

    /// Set the length (seconds) of an already-declared buffer.
    pub fn set_buffer_seconds(&mut self, buf: BufId, seconds: f32) {
        self.buffers[buf.0 as usize] = seconds;
    }

    /// Declared named-buffer lengths in seconds, indexed by [`BufId`].
    pub(crate) fn buffers(&self) -> &[f32] {
        &self.buffers
    }

    /// Mark `node` as a sink: a side-effecting op (e.g. `record`) the compiler must emit even
    /// when nothing consumes its output.
    pub fn add_sink(&mut self, node: NodeId) {
        self.sinks.push(node);
    }

    pub(crate) fn sinks(&self) -> &[NodeId] {
        &self.sinks
    }

    /// Allocate a fresh feedback bus, initially unwritten.
    pub fn new_bus(&mut self) -> BusId {
        let id = BusId(self.bus_sources.len() as u32);
        self.bus_sources.push(None);
        id
    }

    /// A node that reads `bus` delayed by one sample.
    pub fn fb_read(&mut self, bus: BusId) -> NodeId {
        self.push(Node::FbRead { bus })
    }

    /// A node that reads audio input `channel` (the current frame's sample).
    pub fn input(&mut self, channel: u32) -> NodeId {
        self.push(Node::Input { channel })
    }

    /// A node that reads control `lane` from the host's per-block control plane.
    pub fn control(&mut self, lane: u32) -> NodeId {
        self.push(Node::Control { lane })
    }

    /// A node that reads the global sample clock (the executor's windowed `now`).
    pub fn now(&mut self) -> NodeId {
        self.push(Node::Now)
    }

    /// A node selecting output `port` of multi-output `source`.
    pub fn output(&mut self, source: NodeId, port: u32) -> NodeId {
        self.push(Node::Output { source, port })
    }

    /// Designate `node` as the source written into `bus` each sample (read next sample).
    pub fn set_bus_source(&mut self, bus: BusId, node: NodeId) {
        self.bus_sources[bus.0 as usize] = Some(node);
    }

    pub fn bus_sources(&self) -> &[Option<NodeId>] {
        &self.bus_sources
    }

    /// Designate `nodes` as the graph's output channels. A later call replaces an
    /// earlier one.
    pub fn set_outputs(&mut self, nodes: Vec<NodeId>) {
        self.outputs = nodes;
    }

    pub fn outputs(&self) -> &[NodeId] {
        &self.outputs
    }

    /// Declare named parameter `name` with `default`, returning its control lane. Name and
    /// count validation is the front-end's responsibility, like arity on [`Graph::ugen`].
    pub fn add_param(&mut self, name: String, default: f32) -> u32 {
        let lane = (PARAM_BASE + self.params.len()) as u32;
        self.params.push((name, default));
        lane
    }

    /// The declared named parameters, in declaration (= lane) order.
    pub fn params(&self) -> &[(String, f32)] {
        &self.params
    }

    /// The node behind a handle. Public so an external graph front-end (cagire's `arf-forth`)
    /// can read back what it built — e.g. to fold constants or inspect a subgraph during
    /// construction. Borrowed: the compiler's walk visits every node twice, and cloning a
    /// `Node` heap-clones its input list.
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0 as usize]
    }

    /// Reject a malformed graph before it reaches the compiler or the realtime VM. The
    /// serialized graph (see [`Node`]) is the untrusted patch boundary: hand-crafted or corrupt
    /// JSON can carry out-of-range node ids, wrong arities, non-finite constants, or cycles —
    /// each of which panics [`crate::compile::compile`] or the VM's fixed-size scratch on the
    /// audio thread. This one pass rejects them with a message instead. In-process `Graph`-API
    /// callers (a Forth front-end) build well-formed graphs by construction and need not call it.
    pub fn validate(&self) -> Result<(), String> {
        use crate::ugen::{Arity, ListShape, def};
        let n = self.nodes.len();

        // Per node: referenced ids in range, arity matches the row, constants finite.
        for (i, node) in self.nodes.iter().enumerate() {
            match node {
                Node::Const(v) => {
                    if !v.is_finite() {
                        return Err(format!("node {i} is a non-finite constant"));
                    }
                }
                Node::Ugen {
                    ugen,
                    inputs,
                    buffer,
                } => {
                    let def = def(*ugen);
                    for &input in inputs {
                        if input.0 as usize >= n {
                            return Err(format!(
                                "node {i} ({}) references input node {} of {n}",
                                def.name, input.0
                            ));
                        }
                    }
                    // The VM's per-op input scratch is `MAX_CHANNELS` wide (arf/src/vm.rs).
                    if inputs.len() > MAX_CHANNELS {
                        return Err(format!(
                            "node {i} ({}) has {} inputs (cap {MAX_CHANNELS})",
                            def.name,
                            inputs.len()
                        ));
                    }
                    match def.arity {
                        Arity::Fixed(a) => {
                            if inputs.len() != a {
                                return Err(format!(
                                    "node {i} ({}) takes {a} inputs, got {}",
                                    def.name,
                                    inputs.len()
                                ));
                            }
                        }
                        Arity::Variadic => {
                            if inputs.is_empty() {
                                return Err(format!(
                                    "node {i} ({}) needs at least one input",
                                    def.name
                                ));
                            }
                        }
                        Arity::VariadicLed { shape } => {
                            if inputs.is_empty() {
                                return Err(format!(
                                    "node {i} ({}) needs a leading operand",
                                    def.name
                                ));
                            }
                            // Input 0 is the trigger/index operand; inputs 1.. are the list.
                            let list = inputs.len() - 1;
                            let ok = match shape {
                                ListShape::Any => list >= 1,
                                ListShape::OddAtLeastThree => list >= 3 && list % 2 == 1,
                            };
                            if !ok {
                                return Err(format!(
                                    "node {i} ({}) has a {list}-element list that breaks its shape law",
                                    def.name
                                ));
                            }
                        }
                    }
                    if let Some(buf) = buffer
                        && buf.0 as usize >= self.buffers.len()
                    {
                        return Err(format!(
                            "node {i} ({}) references buffer {} of {}",
                            def.name,
                            buf.0,
                            self.buffers.len()
                        ));
                    }
                }
                Node::FbRead { bus } => {
                    if bus.0 as usize >= self.bus_sources.len() {
                        return Err(format!(
                            "node {i} reads feedback bus {} of {}",
                            bus.0,
                            self.bus_sources.len()
                        ));
                    }
                }
                Node::Output { source, port } => {
                    if source.0 as usize >= n {
                        return Err(format!("node {i} output selects node {} of {n}", source.0));
                    }
                    match &self.nodes[source.0 as usize] {
                        Node::Ugen { ugen, .. } => {
                            let outs = def(*ugen).outputs;
                            if *port as usize >= outs {
                                return Err(format!(
                                    "node {i} selects port {port} of a {outs}-output generator"
                                ));
                            }
                        }
                        _ => {
                            return Err(format!(
                                "node {i} output selects node {}, which is not a generator",
                                source.0
                            ));
                        }
                    }
                }
                Node::Input { .. } | Node::Control { .. } | Node::Now => {}
            }
        }

        // Compile roots reference real nodes too (they bypass the per-node loop above).
        for &out in &self.outputs {
            if out.0 as usize >= n {
                return Err(format!("graph output references node {} of {n}", out.0));
            }
        }
        for (b, src) in self.bus_sources.iter().enumerate() {
            if let Some(node) = src
                && node.0 as usize >= n
            {
                return Err(format!("bus {b} source references node {} of {n}", node.0));
            }
        }
        for &sink in &self.sinks {
            if sink.0 as usize >= n {
                return Err(format!("sink references node {} of {n}", sink.0));
            }
        }

        // Buffer lengths and parameter defaults must be finite: serde casts an out-of-range
        // JSON float to `inf` when it deserializes an f32.
        for (b, &secs) in self.buffers.iter().enumerate() {
            if !secs.is_finite() {
                return Err(format!("buffer {b} has a non-finite length"));
            }
        }
        for (name, default) in &self.params {
            if !default.is_finite() {
                return Err(format!("param {name:?} has a non-finite default"));
            }
        }

        // Reject cycles: the compiler's `emit` memoises a node only on its Exit, so a cycle
        // makes its worklist grow without bound (OOM on the control thread). A valid graph is a
        // DAG whose only feedback is through `FbRead` leaves — so this walk follows just `Ugen`
        // inputs and `Output` sources (the edges `emit` recurses), from the compile roots.
        #[derive(Clone, Copy, PartialEq)]
        enum Color {
            White,
            Gray,
            Black,
        }
        enum Step {
            Enter(NodeId),
            Exit(NodeId),
        }
        let mut color = vec![Color::White; n];
        let roots = self
            .outputs
            .iter()
            .copied()
            .chain(self.bus_sources.iter().filter_map(|s| *s))
            .chain(self.sinks.iter().copied());
        let mut work: Vec<Step> = Vec::new();
        for root in roots {
            work.push(Step::Enter(root));
            while let Some(step) = work.pop() {
                match step {
                    Step::Enter(id) => {
                        let idx = id.0 as usize;
                        match color[idx] {
                            Color::Black => continue,
                            Color::Gray => {
                                return Err(format!("graph has a cycle through node {idx}"));
                            }
                            Color::White => {}
                        }
                        color[idx] = Color::Gray;
                        work.push(Step::Exit(id));
                        match &self.nodes[idx] {
                            Node::Ugen { inputs, .. } => {
                                for &input in inputs {
                                    work.push(Step::Enter(input));
                                }
                            }
                            Node::Output { source, .. } => work.push(Step::Enter(*source)),
                            _ => {}
                        }
                    }
                    Step::Exit(id) => color[id.0 as usize] = Color::Black,
                }
            }
        }

        Ok(())
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }
}
