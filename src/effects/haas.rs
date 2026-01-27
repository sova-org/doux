//! Haas effect — short delay on one channel for spatial placement.

const BUFFER_SIZE: usize = 2048;

/// Mono delay line for Haas effect.
#[derive(Clone, Copy)]
pub struct Haas {
    buffer: [f32; BUFFER_SIZE],
    write_pos: usize,
}

impl Default for Haas {
    fn default() -> Self {
        Self {
            buffer: [0.0; BUFFER_SIZE],
            write_pos: 0,
        }
    }
}

impl Haas {
    /// Delays the input by `ms` milliseconds. Returns the delayed sample.
    pub fn process(&mut self, input: f32, ms: f32, sr: f32) -> f32 {
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) & (BUFFER_SIZE - 1);

        let delay_samples = (ms * sr * 0.001).clamp(1.0, (BUFFER_SIZE - 2) as f32);
        let int_part = delay_samples.floor() as usize;
        let frac = delay_samples - int_part as f32;

        let i0 = (self.write_pos + BUFFER_SIZE - int_part) & (BUFFER_SIZE - 1);
        let i1 = (self.write_pos + BUFFER_SIZE - int_part - 1) & (BUFFER_SIZE - 1);

        self.buffer[i0] + frac * (self.buffer[i1] - self.buffer[i0])
    }
}
