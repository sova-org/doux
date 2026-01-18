//! Comb filter with damping.
//!
//! Creates resonant peaks at `freq` and its harmonics by feeding delayed
//! signal back into itself. Damping applies a lowpass in the feedback path,
//! causing higher harmonics to decay faster (Karplus-Strong style).

const BUFFER_SIZE: usize = 2048;

/// Feedback comb filter with one-pole damping.
#[derive(Clone, Copy)]
pub struct Comb {
    buffer: [f32; BUFFER_SIZE],
    write_pos: usize,
    damp_state: f32,
}

impl Default for Comb {
    fn default() -> Self {
        Self {
            buffer: [0.0; BUFFER_SIZE],
            write_pos: 0,
            damp_state: 0.0,
        }
    }
}

impl Comb {
    /// Processes one sample through the comb filter.
    ///
    /// - `freq`: Fundamental frequency (delay = 1/freq)
    /// - `feedback`: Feedback amount `[-0.99, 0.99]`
    /// - `damp`: High-frequency loss per iteration `[0.0, 1.0]`
    ///
    /// Returns the delayed signal (wet only).
    pub fn process(&mut self, input: f32, freq: f32, feedback: f32, damp: f32, sr: f32) -> f32 {
        let delay_samples = (sr / freq).clamp(1.0, (BUFFER_SIZE - 1) as f32);
        let delay_int = delay_samples.floor() as usize;
        let frac = delay_samples - delay_int as f32;

        // Linear interpolation for precise tuning
        let idx0 = (self.write_pos + BUFFER_SIZE - delay_int) & (BUFFER_SIZE - 1);
        let idx1 = (self.write_pos + BUFFER_SIZE - delay_int - 1) & (BUFFER_SIZE - 1);
        let delayed = self.buffer[idx0] + frac * (self.buffer[idx1] - self.buffer[idx0]);

        let feedback = feedback.clamp(-0.99, 0.99);
        let fb_signal = if damp > 0.0 {
            self.damp_state = delayed * (1.0 - damp) + self.damp_state * damp;
            self.damp_state
        } else {
            delayed
        };

        self.buffer[self.write_pos] = input + fb_signal * feedback;
        self.write_pos = (self.write_pos + 1) & (BUFFER_SIZE - 1);

        delayed
    }
}
