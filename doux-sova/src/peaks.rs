use std::sync::atomic::{AtomicU32, Ordering};

const MAX_CHANNELS: usize = 32;

pub struct PeakCapture {
    channels: [AtomicU32; MAX_CHANNELS],
    num_channels: usize,
}

impl PeakCapture {
    pub fn new(num_channels: usize) -> Self {
        assert!(num_channels <= MAX_CHANNELS);
        Self {
            channels: [const { AtomicU32::new(0) }; MAX_CHANNELS],
            num_channels,
        }
    }

    /// Called from audio thread: accumulate per-channel peak from interleaved data.
    #[inline]
    pub fn push(&self, data: &[f32], channels: usize) {
        for frame in data.chunks_exact(channels) {
            for (ch, &sample) in frame.iter().enumerate() {
                if ch >= self.num_channels {
                    break;
                }
                // `abs()` returns a non-negative f32; for non-negative IEEE 754
                // floats, `to_bits()` is monotonic in magnitude, so `fetch_max`
                // on the bit pattern computes the maximum-by-value.
                let abs_bits = sample.abs().to_bits();
                self.channels[ch].fetch_max(abs_bits, Ordering::Relaxed);
            }
        }
    }

    /// Called from reader thread: atomically swap each channel's peak with 0.
    pub fn read_and_reset(&self) -> Vec<f32> {
        let mut peaks = Vec::with_capacity(self.num_channels);
        for ch in 0..self.num_channels {
            let bits = self.channels[ch].swap(0, Ordering::Relaxed);
            peaks.push(f32::from_bits(bits));
        }
        peaks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_read_peaks() {
        let cap = PeakCapture::new(2);
        // Interleaved stereo: L=0.5, R=-0.8, L=0.3, R=0.2
        cap.push(&[0.5, -0.8, 0.3, 0.2], 2);
        let peaks = cap.read_and_reset();
        assert_eq!(peaks.len(), 2);
        assert!((peaks[0] - 0.5).abs() < 1e-6);
        assert!((peaks[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn read_resets() {
        let cap = PeakCapture::new(1);
        cap.push(&[0.9], 1);
        let p1 = cap.read_and_reset();
        assert!((p1[0] - 0.9).abs() < 1e-6);
        let p2 = cap.read_and_reset();
        assert!((p2[0]).abs() < 1e-6);
    }

    #[test]
    fn accumulates_max() {
        let cap = PeakCapture::new(2);
        cap.push(&[0.1, 0.2], 2);
        cap.push(&[0.5, 0.1], 2);
        cap.push(&[0.3, 0.9], 2);
        let peaks = cap.read_and_reset();
        assert!((peaks[0] - 0.5).abs() < 1e-6);
        assert!((peaks[1] - 0.9).abs() < 1e-6);
    }
}
