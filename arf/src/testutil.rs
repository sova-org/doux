//! Test-only graph fixtures, hand-built through the [`Graph`] builder API.
//!
//! These replace the Forth source strings the engine tests once parsed: the Forth front-end
//! now lives in the external `arf-forth` crate, and arf-core is a graph engine that tests
//! itself as one — a patch is a [`Graph`], however a front-end chose to spell it.

use crate::graph::{Graph, NodeId};
use crate::ugen::{self, UGenId};

/// Resolve a ugen word to its id, panicking — a fixture names only real generators.
pub(crate) fn u(name: &str) -> UGenId {
    ugen::lookup(name).unwrap_or_else(|| panic!("no ugen `{name}`"))
}

/// Build a graph inline: the closure adds nodes and returns the output-channel nodes, which
/// become the graph's outputs — the builder mirror of a Forth program ending in `out`.
pub(crate) fn graph_of(build: impl FnOnce(&mut Graph) -> Vec<NodeId>) -> Graph {
    let mut g = Graph::new();
    let outs = build(&mut g);
    g.set_outputs(outs);
    g
}

/// `v out` — a bare DC constant.
pub(crate) fn dc(v: f32) -> Graph {
    graph_of(|g| vec![g.constant(v)])
}

/// `freq <name> out` — a one-input oscillator on a constant frequency (`sine`, `saw`, …).
pub(crate) fn osc(name: &str, freq: f32) -> Graph {
    graph_of(|g| {
        let f = g.constant(freq);
        vec![g.ugen(u(name), vec![f])]
    })
}

/// `freq <name> gain * out` — an oscillator scaled by a constant gain.
pub(crate) fn osc_gain(name: &str, freq: f32, gain: f32) -> Graph {
    graph_of(|g| {
        let f = g.constant(freq);
        let s = g.ugen(u(name), vec![f]);
        let a = g.constant(gain);
        vec![g.ugen(u("*"), vec![s, a])]
    })
}
