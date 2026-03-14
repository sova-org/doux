/// Power-of-2 circular delay buffer with linear-interpolated reads.
#[derive(Clone, Copy)]
pub struct DelayLine<const N: usize> {
    buffer: [f32; N],
    write_pos: usize,
}

impl<const N: usize> Default for DelayLine<N> {
    fn default() -> Self {
        Self {
            buffer: [0.0; N],
            write_pos: 0,
        }
    }
}

impl<const N: usize> DelayLine<N> {
    const MASK: usize = N - 1;

    /// Write a sample and advance the write head.
    #[inline]
    pub fn write(&mut self, sample: f32) {
        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) & Self::MASK;
    }

    /// Read with linear interpolation at `delay_samples` behind the write head.
    #[inline]
    pub fn read(&self, delay_samples: f32) -> f32 {
        let delay_int = delay_samples.floor() as usize;
        let frac = delay_samples - delay_int as f32;
        self.read_frac(delay_int, frac)
    }

    /// Read at an integer delay with fractional interpolation.
    #[inline]
    pub fn read_frac(&self, delay_int: usize, frac: f32) -> f32 {
        let idx0 = (self.write_pos + N - delay_int) & Self::MASK;
        let idx1 = (self.write_pos + N - delay_int - 1) & Self::MASK;
        self.buffer[idx0] + frac * (self.buffer[idx1] - self.buffer[idx0])
    }

    /// Write a sample without advancing (for effects that write before read).
    #[inline]
    pub fn write_at_head(&mut self, sample: f32) {
        self.buffer[self.write_pos] = sample;
    }

    /// Advance write position without writing.
    #[inline]
    pub fn advance(&mut self) {
        self.write_pos = (self.write_pos + 1) & Self::MASK;
    }

    /// Current write position (for effects that need direct buffer access).
    #[inline]
    pub fn pos(&self) -> usize {
        self.write_pos
    }
}
