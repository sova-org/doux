//! Engine metrics: the numbers a consumer (the web app, the nvim statusline) shows to make the
//! engine legible — its size, its memory, how cleanly the last edit carried over, and how hard
//! the CPU is working.
//!
//! Two classes, deliberately kept apart, because only one is comparable across machines:
//!
//! - **Deterministic** ([`Metrics`], [`ReconcileStats`], [`graph_weight`]) — read straight off the
//!   compiled [`Program`] and the reconcile staging. They are properties of the graph, not the
//!   hardware, so they are *bit-identical on every OS and backend* (VM/JIT/AOT/WASM) and exactly
//!   reproducible: the structural sizes, the memory footprint, the graph weight, the reconciliation
//!   report.
//! - **Machine-relative** ([`Perf`], published through [`RtStats`]) — DSP load and dropouts, which
//!   depend on the CPU, the buffer size, and the scheduler. They cannot be made comparable across
//!   machines, only *well-defined*: load is the dimensionless ratio `compute / deadline`, smoothed
//!   by a documented EMA. The engine core stays sample-clocked; the wall-clock measurement is the
//!   driver's job, never the DSP's — so it cannot perturb the bit-exact output.
//!
//! The wire format — the `metrics` / `perf` stdin frames and the worklet's matching posts — is
//! defined once here as JSON, so the native and web frontends surface identical fields.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use serde_json::{json, Value};

use crate::ir::{Op, Program};

/// The summed per-sample cost of a program: Σ over its ops of the producing UGen's
/// [`cost`](crate::ugen::UGen::cost), counting each non-UGen leaf (`Const`, a bus/input/clock read)
/// as one unit. A machine-independent estimate of how much arithmetic one frame costs — it ranks
/// two patches identically on any CPU, unlike [`Perf::load`]. An AOT engine's IR carries no ops (the
/// DSP lives in the linked object), so its weight reads 0.
pub fn graph_weight(program: &Program) -> u32 {
    program
        .ops()
        .iter()
        .map(|op| match op {
            Op::Ugen { ugen, .. } => crate::ugen::def(*ugen).cost as u32,
            _ => 1,
        })
        .sum()
}

/// The deterministic snapshot of a compiled program: its structural sizes, its memory footprint,
/// and its graph weight. Built with [`Metrics::of`]; serialized (with the optional
/// [`ReconcileStats`] of the swap that installed it) into the `metrics` frame each frontend emits
/// on a successful send.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    /// IR ops (the flat program length).
    pub ops: u32,
    /// Per-sample register scratch slots.
    pub registers: u32,
    /// Persistent per-UGen state slots (oscillator phase, filter memory, …).
    pub state_slots: u32,
    /// Feedback-bus slots.
    pub buses: u32,
    /// Observation taps (`scope` points).
    pub taps: u32,
    /// Voice-pool size (polyphony; 1 = mono).
    pub voices: u32,
    /// Device-routed audio output channels.
    pub audio_channels: u32,
    /// Audio input channels the program reads.
    pub in_channels: u32,
    /// Sample-memory arena, in bytes (delay lines, wavetables, the reverb tank).
    pub buffer_bytes: u32,
    /// Total engine memory across the four planes, in bytes: `4·(registers + state + buses + buffer)`.
    pub memory_bytes: u32,
    /// The [`graph_weight`].
    pub graph_weight: u32,
    /// The rate the program was compiled for (Hz).
    pub sample_rate: f32,
}

impl Metrics {
    /// Read the deterministic figures off a compiled program. Pure: no clock, no I/O, no globals,
    /// so the result is identical on every backend that runs this program.
    pub fn of(program: &Program) -> Self {
        let registers = program.num_registers();
        let state = program.state_len();
        let buses = program.bus_len();
        let buffer = program.buffer_len();
        // The four memory planes are all f32 (4 bytes each): the per-sample register scratch, the
        // persistent state, the feedback buses, and the sample-memory arena. One reproducible figure.
        Metrics {
            ops: program.ops().len() as u32,
            registers: registers as u32,
            state_slots: state as u32,
            buses: buses as u32,
            taps: program.tap_names().len() as u32,
            voices: program.voice_count() as u32,
            audio_channels: program.audio_channels() as u32,
            in_channels: program.in_channels() as u32,
            buffer_bytes: (4 * buffer) as u32,
            memory_bytes: (4 * (registers + state + buses + buffer)) as u32,
            graph_weight: graph_weight(program),
            sample_rate: program.sample_rate,
        }
    }

    /// The `metrics` frame payload: the structural figures, plus the reconciliation report when a
    /// swap produced one. Defined here so the native stdout frame and the worklet post never drift.
    pub fn to_json(self, reconcile: Option<&ReconcileStats>) -> Value {
        json!({
            "ops": self.ops,
            "registers": self.registers,
            "stateSlots": self.state_slots,
            "buses": self.buses,
            "taps": self.taps,
            "voices": self.voices,
            "audioChannels": self.audio_channels,
            "inChannels": self.in_channels,
            "bufferBytes": self.buffer_bytes,
            "memoryBytes": self.memory_bytes,
            "graphWeight": self.graph_weight,
            "sampleRate": self.sample_rate,
            "reconcile": reconcile.copied().map(ReconcileStats::to_json),
        })
    }
}

/// The reconciliation report for one hot-swap: how much of the running engine the edit carried
/// forward rather than restarting. This is arf's signature metric — high reuse is *why* a live edit
/// does not click. Computed by a host's hot-swap reconciliation, where both the old and new
/// programs' layouts are known.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReconcileStats {
    /// Stateful ops in the new program (oscillator phases, filter memories, envelope clocks, …).
    pub stateful_total: u32,
    /// How many of them matched an op of the displaced program and carried their state over.
    pub stateful_reused: u32,
    /// Total f32 state/bus scalars carried (the sum of the matched ops' slot counts).
    pub scalars_carried: u32,
    /// Buffer-bearing ops (delay lines, wavetables, the reverb tank) in the new program.
    pub buffers_total: u32,
    /// Buffers carried whole by O(1) arena donation — an instant swap with an unchanged layout.
    pub buffers_donated: u32,
    /// Buffer regions copied through a crossfade — a faded swap, which cannot donate.
    pub regions_copied: u32,
    /// Matched regions too large to copy within the fade budget — reset to silence.
    pub regions_declined: u32,
}

impl ReconcileStats {
    fn to_json(self) -> Value {
        json!({
            "statefulTotal": self.stateful_total,
            "statefulReused": self.stateful_reused,
            "scalarsCarried": self.scalars_carried,
            "buffersTotal": self.buffers_total,
            "buffersDonated": self.buffers_donated,
            "regionsCopied": self.regions_copied,
            "regionsDeclined": self.regions_declined,
        })
    }
}

/// The averaging window for [`Perf::load_avg`], in wall-seconds — the same on every machine. The
/// per-callback EMA coefficient `α = block_seconds / τ` follows from it, so the smoothing tracks
/// roughly one second of audio whatever the buffer size or sample rate.
const LOAD_TAU_SECS: f32 = 1.0;

/// A reader's snapshot of [`RtStats`]: the machine-relative performance figures, taken on the
/// control thread and serialized into the `perf` frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Perf {
    /// Instantaneous DSP load of the last callback, `compute / deadline`. 1.0 = the whole budget;
    /// above 1.0 is an overrun (a dropout).
    pub load: f32,
    /// The load smoothed over ~[`LOAD_TAU_SECS`] — the steady "average" reading.
    pub load_avg: f32,
    /// The peak load since this snapshot was last taken (reset on read, like a peak-hold meter).
    pub load_peak: f32,
    /// Callbacks that overran their deadline (`load > 1`) since the stream started.
    pub xruns: u64,
    /// Frames rendered since the stream started (`frames / sampleRate` = uptime in seconds).
    pub frames_rendered: u64,
    /// The last callback's size in frames — the device buffer (`frames / sampleRate` = latency).
    pub block_frames: u32,
    /// The transport tempo in BPM (the runtime musical clock).
    pub bpm: f32,
}

impl Perf {
    /// The `perf` frame payload. `sample_rate` turns the frame/sample figures into the seconds and
    /// milliseconds a consumer displays — derived here, in one place.
    pub fn to_json(self, sample_rate: f32) -> Value {
        let sr = sample_rate.max(1.0);
        json!({
            "load": self.load,
            "loadAvg": self.load_avg,
            "loadPeak": self.load_peak,
            "xruns": self.xruns,
            "framesRendered": self.frames_rendered,
            "uptimeSeconds": self.frames_rendered as f64 / sr as f64,
            "blockFrames": self.block_frames,
            "latencyMs": self.block_frames as f32 / sr * 1000.0,
            "bpm": self.bpm,
        })
    }
}

/// Lock-free realtime-performance counters: the audio thread is the sole writer (one
/// [`observe`](RtStats::observe) per callback), a control thread the sole reader
/// ([`snapshot`](RtStats::snapshot)). Every access is `Relaxed` — the figures are advisory, ordered
/// against nothing, and never fed back into the DSP, so they cannot perturb the bit-exact output.
/// Floats are stored as bit patterns; the peak
/// uses `fetch_max` on the pattern, which equals max-on-value because the load is always ≥ 0 (and
/// non-negative f32 bit patterns order like their values).
pub struct RtStats {
    load: AtomicU32,
    avg: AtomicU32,
    peak: AtomicU32,
    xruns: AtomicU64,
    frames_rendered: AtomicU64,
    block_frames: AtomicU32,
    bpm: AtomicU32,
}

impl RtStats {
    pub fn new() -> Self {
        RtStats {
            load: AtomicU32::new(0),
            avg: AtomicU32::new(0),
            peak: AtomicU32::new(0),
            xruns: AtomicU64::new(0),
            frames_rendered: AtomicU64::new(0),
            block_frames: AtomicU32::new(0),
            bpm: AtomicU32::new(0),
        }
    }

    /// Publish one callback's timing. `compute_secs` is the wall-time the DSP took; the deadline is
    /// `frames / sample_rate` (the time the device gives back per buffer), so `load = compute /
    /// deadline` is dimensionless and comparable across buffer sizes. Realtime-safe: a handful of
    /// relaxed atomic stores, no allocation — sound inside `assert_no_alloc`. The sole writer, so
    /// the load/avg read-modify-write needs no compare-and-swap.
    pub fn observe(&self, frames: u32, compute_secs: f32, sample_rate: f32, frames_rendered: u64, bpm: f32) {
        let deadline = frames as f32 / sample_rate.max(1.0);
        let load = if deadline > 0.0 { compute_secs / deadline } else { 0.0 };
        self.load.store(load.to_bits(), Ordering::Relaxed);
        // EMA toward the current load with a wall-time-constant window: α = block_seconds / τ, so
        // the same ~1 s smoothing holds on any machine regardless of the buffer size.
        let prev = f32::from_bits(self.avg.load(Ordering::Relaxed));
        let alpha = (deadline / LOAD_TAU_SECS).clamp(0.0, 1.0);
        self.avg.store((prev + alpha * (load - prev)).to_bits(), Ordering::Relaxed);
        // Peak-hold since the last read (fetch_max on the pattern == max on the value, load ≥ 0).
        self.peak.fetch_max(load.to_bits(), Ordering::Relaxed);
        if load > 1.0 {
            self.xruns.fetch_add(1, Ordering::Relaxed);
        }
        self.frames_rendered.store(frames_rendered, Ordering::Relaxed);
        self.block_frames.store(frames, Ordering::Relaxed);
        self.bpm.store(bpm.to_bits(), Ordering::Relaxed);
    }

    /// Read the counters, resetting the windowed peak (peak-hold since the previous snapshot).
    pub fn snapshot(&self) -> Perf {
        Perf {
            load: f32::from_bits(self.load.load(Ordering::Relaxed)),
            load_avg: f32::from_bits(self.avg.load(Ordering::Relaxed)),
            load_peak: f32::from_bits(self.peak.swap(0, Ordering::Relaxed)),
            xruns: self.xruns.load(Ordering::Relaxed),
            frames_rendered: self.frames_rendered.load(Ordering::Relaxed),
            block_frames: self.block_frames.load(Ordering::Relaxed),
            bpm: f32::from_bits(self.bpm.load(Ordering::Relaxed)),
        }
    }
}

impl Default for RtStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::compile;
    use crate::graph::Graph;
    use crate::testutil::{graph_of, osc_gain, u};

    fn prog(graph: &Graph) -> Program {
        compile(graph, 48_000.0)
    }

    /// `440 sine 0.2 *  0.4 0.9 0.5 verb  out` — a reverb hung off a scaled sine.
    fn sine_verb() -> Graph {
        graph_of(|g| {
            let f = g.constant(440.0);
            let s = g.ugen(u("sine"), vec![f]);
            let a = g.constant(0.2);
            let dry = g.ugen(u("*"), vec![s, a]);
            let (room, damp, mix) = (g.constant(0.4), g.constant(0.9), g.constant(0.5));
            vec![g.ugen(u("verb"), vec![dry, room, damp, mix])]
        })
    }

    #[test]
    fn graph_weight_sums_costs_and_ranks_patches() {
        // A sine-bearing patch weighs at least the sine's cost; a reverb patch outweighs it — the
        // ranking is what makes the figure useful, and it is the same on every backend.
        let light = graph_weight(&prog(&osc_gain("sine", 440.0, 0.2)));
        let heavy = graph_weight(&prog(&sine_verb()));
        assert!(light >= 12, "a sine patch weighs at least the sine cost: {light}");
        assert!(heavy > light, "a reverb patch outweighs a bare sine: {heavy} vs {light}");
    }

    #[test]
    fn metrics_of_reads_program_sizes() {
        let m = Metrics::of(&prog(&osc_gain("sine", 440.0, 0.2)));
        assert_eq!(m.sample_rate, 48_000.0);
        assert_eq!(m.audio_channels, 1);
        assert!(m.ops >= 4, "const, sine, const, mul at least: {}", m.ops);
        assert!(m.state_slots >= 1, "the sine carries a phase slot");
        // The footprint identity holds (buffer_bytes is 4·buffer, so the division is exact).
        assert_eq!(m.memory_bytes, 4 * (m.registers + m.state_slots + m.buses) + m.buffer_bytes);
    }

    #[test]
    fn rtstats_load_peak_and_xruns() {
        let rt = RtStats::new();
        // 128 frames at 48 kHz is a 2.667 ms deadline; 1.333 ms of compute is half the budget.
        rt.observe(128, 0.5 * 128.0 / 48_000.0, 48_000.0, 128, 120.0);
        let p = rt.snapshot();
        assert!((p.load - 0.5).abs() < 1e-4, "load {}", p.load);
        assert!((p.load_peak - 0.5).abs() < 1e-4, "peak {}", p.load_peak);
        assert_eq!(p.xruns, 0);
        assert_eq!(rt.snapshot().load_peak, 0.0, "the peak resets on read");
        // An overrun (load > 1) counts as a dropout.
        rt.observe(128, 2.0 * 128.0 / 48_000.0, 48_000.0, 256, 120.0);
        assert_eq!(rt.snapshot().xruns, 1, "load > 1 is an xrun");
    }
}
