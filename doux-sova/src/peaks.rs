use std::sync::atomic::{AtomicU8, Ordering};

const MAX_CHANNELS: usize = 32;

pub struct PeakCapture {
    buffers: [Box<[f32; MAX_CHANNELS]>; 2],
    num_channels: usize,
    write_idx: AtomicU8,
}

unsafe impl Send for PeakCapture {}
unsafe impl Sync for PeakCapture {}

impl PeakCapture {
    pub fn new(num_channels: usize) -> Self {
        assert!(num_channels <= MAX_CHANNELS);
        Self {
            buffers: [Box::new([0.0; MAX_CHANNELS]), Box::new([0.0; MAX_CHANNELS])],
            num_channels,
            write_idx: AtomicU8::new(0),
        }
    }

    /// Called from audio thread: accumulate per-channel peak from interleaved data.
    #[inline]
    pub fn push(&self, data: &[f32], channels: usize) {
        let buf_idx = self.write_idx.load(Ordering::Relaxed) as usize;
        let buf_ptr = self.buffers[buf_idx].as_ptr() as *mut f32;
        for frame in data.chunks_exact(channels) {
            for (ch, &sample) in frame.iter().enumerate() {
                if ch >= self.num_channels {
                    break;
                }
                // SAFETY: single writer (audio thread), ch < MAX_CHANNELS
                unsafe {
                    let ptr = buf_ptr.add(ch);
                    let current = *ptr;
                    let abs = sample.abs();
                    if abs > current {
                        *ptr = abs;
                    }
                }
            }
        }
    }

    /// Called from reader thread: swap buffers, return accumulated peaks, zero consumed buffer.
    pub fn read_and_reset(&self) -> Vec<f32> {
        let old = self.write_idx.load(Ordering::Relaxed);
        let new = 1 - old;
        // Zero the new write buffer before swapping
        let new_ptr = self.buffers[new as usize].as_ptr() as *mut f32;
        for i in 0..self.num_channels {
            unsafe {
                *new_ptr.add(i) = 0.0;
            }
        }
        self.write_idx.store(new, Ordering::Release);

        // Read accumulated peaks from old write buffer
        let old_ptr = self.buffers[old as usize].as_ptr();
        let mut peaks = Vec::with_capacity(self.num_channels);
        for i in 0..self.num_channels {
            peaks.push(unsafe { *old_ptr.add(i) });
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
