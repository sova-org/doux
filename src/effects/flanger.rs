//! Flanger effect with LFO-modulated delay.
//!
//! Creates the characteristic "jet plane" sweep by mixing the input with a
//! short, modulated delay (0.5-10ms). Feedback intensifies the comb filtering.

use crate::oscillator::Phasor;

const BUFFER_SIZE: usize = 512;
const MIN_DELAY_MS: f32 = 0.5;
const MAX_DELAY_MS: f32 = 10.0;
const DELAY_RANGE_MS: f32 = MAX_DELAY_MS - MIN_DELAY_MS;

/// Mono flanger with feedback.
#[derive(Clone, Copy)]
pub struct Flanger {
    buffer: [f32; BUFFER_SIZE],
    write_pos: usize,
    lfo: Phasor,
    feedback: f32,
}

impl Default for Flanger {
    fn default() -> Self {
        Self {
            buffer: [0.0; BUFFER_SIZE],
            write_pos: 0,
            lfo: Phasor::default(),
            feedback: 0.0,
        }
    }
}

impl Flanger {
    /// Processes one sample.
    ///
    /// - `rate`: LFO speed in Hz (typical: 0.1-2.0)
    /// - `depth`: Modulation amount `[0.0, 1.0]` (squared for smoother response)
    /// - `feedback`: Resonance `[0.0, 0.95]`
    ///
    /// Returns 50/50 dry/wet mix.
    pub fn process(
        &mut self,
        input: f32,
        rate: f32,
        depth: f32,
        feedback: f32,
        sr: f32,
        isr: f32,
    ) -> f32 {
        let lfo_val = self.lfo.sine(rate, isr);
        let depth_curve = depth * depth;
        let delay_ms = MIN_DELAY_MS + depth_curve * DELAY_RANGE_MS * (lfo_val * 0.5 + 0.5);

        let delay_samples = (delay_ms * sr * 0.001).clamp(1.0, BUFFER_SIZE as f32 - 2.0);

        let read_pos_int = delay_samples.floor() as usize;
        let frac = delay_samples - read_pos_int as f32;

        let read_index1 = (self.write_pos + BUFFER_SIZE - read_pos_int) & (BUFFER_SIZE - 1);
        let read_index2 = (self.write_pos + BUFFER_SIZE - read_pos_int - 1) & (BUFFER_SIZE - 1);

        let delayed1 = self.buffer[read_index1];
        let delayed2 = self.buffer[read_index2];
        let delayed = delayed1 + frac * (delayed2 - delayed1);

        let feedback = feedback.clamp(0.0, 0.95);

        self.buffer[self.write_pos] = input + self.feedback * feedback;
        self.write_pos = (self.write_pos + 1) & (BUFFER_SIZE - 1);

        self.feedback = delayed;

        input * 0.5 + delayed * 0.5
    }
}
