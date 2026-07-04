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
//! - [`engine`]  — a program paired with its VM state; the unit handed to the audio thread.
//!
//! Front-ends live outside the engine: the Forth patch language is the `arf-forth` crate,
//! and a host may build a [`graph::Graph`] any other way. This crate is just the engine.

pub mod compile;
pub mod engine;
// The realtime hot-swap loop + crossfade, surfaced through `engine` (`engine::EngineHost`).
mod enginehost;
pub mod graph;
pub mod ir;
// Engine metrics: deterministic structural / reconciliation figures (exact on every backend)
// and the machine-relative realtime-performance block. Cross-target — only the wall-clock
// perf timing is a driver's job.
pub mod metrics;
pub mod reconcile;
pub mod ugen;
pub mod vm;

// Device-boundary NaN/inf scrubbing, surfaced through `engine` (`engine::sanitize_block`). A leaf
// module so the speaker-protection guard has one home, separate from the executor.
mod sanitize;

// MIDI message parsing, shared by every host front-end (fed raw bytes).
pub mod midi;

// Realtime-safe MIDI voice allocation: maps notes onto control-plane voice slots. Pure over
// the control-lane constants, so every host shares it.
pub mod voices;

// Test-only graph fixtures, hand-built via the `graph` builder API (see the module docs).
#[cfg(test)]
mod testutil;
