//! Per-voice feedback delay.
//!
//! Re-injects the voice's output back into itself with a controllable delay
//! time and damping. Enables slapback echoes, metallic resonances, and
//! short rhythmic feedback loops.

const BUFFER_SIZE: usize = 32768;

/// Feedback delay with one-pole damping in the feedback path.
#[derive(Clone, Copy)]
pub struct Feedback {
    buffer: [f32; BUFFER_SIZE],
    write_pos: usize,
    damp_state: f32,
}

impl Default for Feedback {
    fn default() -> Self {
        Self {
            buffer: [0.0; BUFFER_SIZE],
            write_pos: 0,
            damp_state: 0.0,
        }
    }
}

impl Feedback {
    /// Processes one sample through the feedback delay.
    ///
    /// - `feedback`: Re-injection level `[0.0, 0.99]`
    /// - `time_ms`: Delay time in milliseconds
    /// - `damp`: High-frequency loss in feedback path `[0.0, 1.0]`
    ///
    /// Returns 50/50 dry/wet mix.
    pub fn process(&mut self, input: f32, feedback: f32, time_ms: f32, damp: f32, sr: f32) -> f32 {
        let delay_samples = (time_ms * sr * 0.001).clamp(1.0, (BUFFER_SIZE - 1) as f32);
        let delay_int = delay_samples.floor() as usize;
        let frac = delay_samples - delay_int as f32;

        let idx0 = (self.write_pos + BUFFER_SIZE - delay_int) & (BUFFER_SIZE - 1);
        let idx1 = (self.write_pos + BUFFER_SIZE - delay_int - 1) & (BUFFER_SIZE - 1);
        let delayed = self.buffer[idx0] + frac * (self.buffer[idx1] - self.buffer[idx0]);

        let feedback = feedback.clamp(0.0, 0.99);
        let fb_signal = if damp > 0.0 {
            self.damp_state = delayed * (1.0 - damp) + self.damp_state * damp;
            self.damp_state
        } else {
            delayed
        };

        self.buffer[self.write_pos] = input + fb_signal * feedback;
        self.write_pos = (self.write_pos + 1) & (BUFFER_SIZE - 1);

        input * 0.5 + delayed * 0.5
    }
}
