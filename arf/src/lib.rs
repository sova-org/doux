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
//! - [`vm`]      — the reference VM that runs a program one sample at a time.
//!   It makes sound today and is the correctness oracle a future Cranelift JIT
//!   backend will be validated against.
//!
//! Front-ends live outside the engine: the Forth patch language is the `arf-forth` crate,
//! and a host may build a [`graph::Graph`] any other way. This crate is just the engine.

pub mod compile;
pub mod graph;
pub mod ir;
// Engine metrics: deterministic structural / reconciliation figures (exact on every backend)
// and the machine-relative realtime-performance block. Cross-target — only the wall-clock
// perf timing is a driver's job.
pub mod metrics;
pub mod ugen;
pub mod vm;

// Test-only graph fixtures, hand-built via the `graph` builder API (see the module docs).
#[cfg(test)]
mod testutil;
