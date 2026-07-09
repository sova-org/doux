//! The VM: runs a [`Program`] one sample at a time, on every target (native and wasm).
//!
//! Registers are scratch (recomputed every sample); state persists across samples.

use crate::ir::{Op, Program};
use crate::ugen;

pub struct Vm {
    regs: Vec<f32>,
    state: Vec<f32>,
    buses: Vec<f32>,
    buffers: Vec<f32>,
}

impl Vm {
    pub fn new(program: &Program) -> Self {
        let mut state = vec![0.0; program.state_len()];
        // Apply the sparse fresh-init seeds (noise counters) over the zero fill so co-existing
        // noise sources decorrelate.
        for &(slot, v) in program.initial_state() {
            state[slot as usize] = v;
        }
        Vm {
            regs: vec![0.0; program.num_registers()],
            state,
            buses: vec![0.0; program.bus_len()],
            buffers: vec![0.0; program.buffer_len()],
        }
    }

    /// Reset to the fresh state `Vm::new(program)` would produce, reusing the arenas — no
    /// allocation, so a pooled instance can be recycled off the audio thread between uses.
    /// `seed_offset` scatters the program's fresh-init seeds (the noise counters) through the
    /// shared avalanche so concurrent instances of one program decorrelate; `0` reproduces
    /// `Vm::new` bit-for-bit. Deterministic per `(program, seed_offset)` pair.
    pub fn reset(&mut self, program: &Program, seed_offset: u32) {
        // Registers are per-sample scratch — every op rewrites its block before any read
        // (the topological invariant), so only the persistent planes need clearing.
        self.state.fill(0.0);
        self.buses.fill(0.0);
        self.buffers.fill(0.0);
        for &(slot, v) in program.initial_state() {
            // The seed values are exact integers < 2^24 (see `ugen::noise_seed`), so the
            // round-trip through f32 is lossless and re-hashing the offset sum is exact.
            let v = if seed_offset == 0 {
                v
            } else {
                ugen::noise_seed((v as u32).wrapping_add(seed_offset))
            };
            self.state[slot as usize] = v;
        }
    }

    /// Compute one frame, advancing internal state (phases, filter memory). `frame_pos` is the
    /// frame's absolute position in the global sample clock (the executor advances it one per
    /// frame); it surfaces to UGens as the windowed `now`. Reads one value per input channel from
    /// `input` (`input.len()` must be `program.in_channels`), reads control values from `control`
    /// (`control.len()` must be `program.control_len`; the same slice is passed for every frame of
    /// a block — the plane is frame-invariant), and writes one per output channel into `frame`
    /// (`frame.len()` is the channel count).
    pub fn tick_frame(
        &mut self,
        program: &Program,
        frame_pos: u64,
        input: &[f32],
        control: &[f32],
        frame: &mut [f32],
    ) {
        let sr = program.sample_rate;
        // The pure-core face of time: the absolute position reduced into the precision window,
        // computed once per frame and shared by the `now` leaf and every UGen's `TickCtx.now`.
        let now = (frame_pos & (crate::ir::NOW_WINDOW - 1)) as f32;
        // Split the borrows: registers are read-then-written, state is persistent.
        let regs = &mut self.regs;
        let state = &mut self.state;
        let buses = &mut self.buses;
        let buffers = &mut self.buffers;
        let arena = program.inputs();
        // Per-op input gather and output scratch, stack-allocated so the tick path never
        // touches the heap. Inputs are sized to the widest possible op: a fixed generator is
        // bounded by MAX_ARITY, but a variadic one (e.g. `mix`) consumes a whole channel-list,
        // so the bound is MAX_CHANNELS.
        let mut scratch = [0.0_f32; crate::graph::MAX_CHANNELS];
        let mut out_scratch = [0.0_f32; ugen::MAX_OUTPUTS];

        // The registers an op writes are the contiguous block starting at `reg_cursor`, which
        // advances in op order: a leaf writes one, a generator writes its `outputs`.
        let mut reg_cursor = 0usize;
        for op in program.ops().iter() {
            match *op {
                Op::Const(v) => {
                    regs[reg_cursor] = v;
                    reg_cursor += 1;
                }
                // Read the bus's stored value from last sample (the one-sample delay).
                Op::FbRead { slot } => {
                    regs[reg_cursor] = buses[slot as usize];
                    reg_cursor += 1;
                }
                // Read this frame's audio input (channel < in_channels by construction).
                Op::Input { channel } => {
                    regs[reg_cursor] = input[channel as usize];
                    reg_cursor += 1;
                }
                // Read a control-plane lane (lane < control_len by construction). The plane is
                // latched per block, so this is the same value for every frame of the block.
                Op::Control { lane } => {
                    regs[reg_cursor] = control[lane as usize];
                    reg_cursor += 1;
                }
                // Read the sample clock — frame-strided (advances per sample), unlike `Control`.
                Op::Now => {
                    regs[reg_cursor] = now;
                    reg_cursor += 1;
                }
                // Inlined glue arithmetic — the most numerous op kind in a patch; matched
                // here so it pays none of the UGen call apparatus below.
                Op::Bin { kind, a, b } => {
                    let x = regs[a.0 as usize];
                    let y = regs[b.0 as usize];
                    regs[reg_cursor] = match kind {
                        crate::ir::BinKind::Add => x + y,
                        crate::ir::BinKind::Sub => x - y,
                        crate::ir::BinKind::Mul => x * y,
                        crate::ir::BinKind::Div => x / y,
                    };
                    reg_cursor += 1;
                }
                Op::Ugen { def, input_start, input_count, state_base, buffer_base, buffer_len } => {
                    let start = input_start as usize;
                    let n_in = input_count as usize;
                    for (k, slot) in scratch[..n_in].iter_mut().enumerate() {
                        *slot = regs[arena[start + k].0 as usize];
                    }
                    let s = state_base as usize;
                    let b = buffer_base as usize;
                    let blen = buffer_len as usize;
                    let mut ctx = ugen::TickCtx {
                        inputs: &scratch[..n_in],
                        state: &mut state[s..s + def.state_slots],
                        buffer: &mut buffers[b..b + blen],
                        sr,
                        now,
                    };
                    (def.tick)(&mut ctx, &mut out_scratch[..def.outputs]);
                    regs[reg_cursor..reg_cursor + def.outputs]
                        .copy_from_slice(&out_scratch[..def.outputs]);
                    reg_cursor += def.outputs;
                }
            }
        }

        // End of frame: store each feedback source into its bus for next sample's `FbRead`.
        for fb in program.feedbacks() {
            buses[fb.slot as usize] = regs[fb.source.0 as usize];
        }

        for (slot, reg) in frame.iter_mut().zip(program.outputs()) {
            *slot = regs[reg.0 as usize];
        }
    }
}

/// Render `frames` frames offline from the start of the clock (frame position 0), interleaved
/// by channel (`frames * channels` samples). Used by tests and (later) offline rendering.
pub fn render(program: &Program, frames: usize) -> Vec<f32> {
    render_from(program, frames, 0)
}

/// Render `frames` frames offline as if the global clock were at `start_pos` at the first frame,
/// interleaved by channel. The harness drives a nonzero start to exercise the `now` clock past
/// frame 0 and across the [`crate::ir::NOW_WINDOW`] wrap — coverage [`render`] (start 0) cannot give.
pub fn render_from(program: &Program, frames: usize, start_pos: u64) -> Vec<f32> {
    let channels = program.outputs().len();
    let in_ch = program.in_channels();
    let mut vm = Vm::new(program);
    let input = vec![0.0; frames * in_ch]; // offline render: input is silence
    let control = vec![0.0; program.control_len()]; // offline render: zeroed control plane (frame-invariant)
    let mut out = vec![0.0; frames * channels];
    for (f, frame) in out.chunks_mut(channels).enumerate() {
        let frame_pos = start_pos + f as u64;
        vm.tick_frame(program, frame_pos, &input[f * in_ch..f * in_ch + in_ch], &control, frame);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::compile;
    use crate::graph::Graph;
    use crate::testutil::{graph_of, osc, osc_gain, u};

    fn program(graph: &Graph, sr: f32) -> Program {
        compile(graph, sr)
    }

    #[test]
    fn constant_expression_is_dc() {
        let p = program(
            &graph_of(|g| {
                let a = g.constant(1.0);
                let b = g.constant(2.0);
                vec![g.ugen(u("+"), vec![a, b])]
            }),
            48_000.0,
        );
        let out = render(&p, 16);
        assert!(out.iter().all(|&s| (s - 3.0).abs() < 1e-6));
    }

    #[test]
    fn sine_amplitude_is_unity() {
        let sr = 48_000.0;
        let p = program(&osc("sine", 440.0), sr);
        let out = render(&p, sr as usize); // one second
        let max = out.iter().cloned().fold(f32::MIN, f32::max);
        let min = out.iter().cloned().fold(f32::MAX, f32::min);
        assert!((max - 1.0).abs() < 1e-2, "max = {max}");
        assert!((min + 1.0).abs() < 1e-2, "min = {min}");
    }

    #[test]
    fn sine_frequency_is_correct() {
        let sr = 48_000.0;
        let p = program(&osc("sine", 440.0), sr);
        let out = render(&p, sr as usize); // one second
        // Count rising zero-crossings; for a 440 Hz tone over 1 s that is ~440.
        let crossings = out
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count();
        assert!((crossings as i32 - 440).abs() <= 1, "crossings = {crossings}");
    }

    #[test]
    fn mul_scales_amplitude() {
        let sr = 48_000.0;
        let p = program(&osc_gain("sine", 440.0, 0.25), sr);
        let out = render(&p, sr as usize);
        let max = out.iter().cloned().fold(f32::MIN, f32::max);
        assert!((max - 0.25).abs() < 1e-2, "max = {max}");
    }

    #[test]
    fn delay_shifts_a_signal_in_time() {
        let sr = 48_000.0;
        // A constant 1.0 through a 10-sample delay: the zero-initialised line reads
        // silence for the first 10 samples, then the constant arrives. The delay time is
        // truncated to whole samples: 0.00022 * 48000 = 10.56 -> 10.
        let p = program(
            &graph_of(|g| {
                let s = g.constant(1.0);
                let t = g.constant(0.00022);
                vec![g.ugen(u("delay"), vec![s, t])]
            }),
            sr,
        );
        let out = render(&p, 20);
        assert!(out[..10].iter().all(|&s| s == 0.0), "empty line first: {out:?}");
        assert!(out[10..].iter().all(|&s| s == 1.0), "then the signal: {out:?}");
    }

    #[test]
    fn reset_matches_a_fresh_vm_and_offsets_decorrelate() {
        let sr = 48_000.0;
        let p = program(&graph_of(|g| vec![g.ugen(u("noise"), vec![])]), sr);
        let channels = p.outputs().len();
        let control = vec![0.0; p.control_len()];
        let run = |vm: &mut Vm| -> Vec<f32> {
            let mut out = vec![0.0; 64 * channels];
            for (f, frame) in out.chunks_mut(channels).enumerate() {
                vm.tick_frame(&p, f as u64, &[], &control, frame);
            }
            out
        };
        let mut vm = Vm::new(&p);
        let fresh = run(&mut vm);
        // Offset 0 restores the fresh stream bit-for-bit after the state advanced.
        vm.reset(&p, 0);
        assert_eq!(run(&mut vm), fresh, "reset(0) must reproduce Vm::new exactly");
        // A nonzero offset reseeds the counter, reading a different region of the
        // shared noise sequence — concurrent instances of one program decorrelate.
        vm.reset(&p, 1);
        assert_ne!(run(&mut vm), fresh, "distinct offsets must yield distinct streams");
    }

    #[test]
    fn lpf_smooths_a_constant_step() {
        // A constant 1.0 through a low cutoff: output rises from ~0 toward 1.0.
        let sr = 48_000.0;
        let p = program(
            &graph_of(|g| {
                let s = g.constant(1.0);
                let c = g.constant(200.0);
                vec![g.ugen(u("lpf"), vec![s, c])]
            }),
            sr,
        );
        let out = render(&p, sr as usize);
        assert!(out[0] < 0.5, "first sample should lag: {}", out[0]);
        assert!(*out.last().unwrap() > 0.99, "should settle near 1.0");
        // Monotonic non-decreasing toward the target.
        assert!(out[100] > out[0]);
    }
}
