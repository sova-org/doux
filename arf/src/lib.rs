//! arf — a live-codable graph audio engine: the pure core, embedded by doux.
//!
//! The data flow is a small pipeline:
//!
//! ```text
//! Graph ──compile──▶ Program ──run──▶ samples
//! (graph)          (ir/compile)      (vm)
//! ```
//!
//! - [`ugen`]    — the UGen contract: every generator declared once in one table.
//! - [`graph`]   — the authoring representation: a DAG of audio nodes.
//! - [`ir`]      — the execution representation: a flat, topologically ordered program.
//! - [`compile`] — lowers a [`graph::Graph`] into an [`ir::Program`].
//! - [`vm`]      — the VM that runs a program one sample at a time.
//!
//! Front-ends live outside the engine: the Forth patch language is cagire's `arf-forth`
//! crate, and a host may build a [`graph::Graph`] any other way. This crate is just the
//! engine; doux compiles and runs the graphs, it never parses a patch language.

pub mod compile;
pub mod graph;
pub mod ir;
pub mod ugen;
pub mod vm;

// Test-only graph fixtures, hand-built via the `graph` builder API (see the module docs).
#[cfg(test)]
mod testutil;
