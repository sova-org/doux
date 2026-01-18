//! Lock-free oscilloscope capture for the audio engine.

use std::sync::atomic::{AtomicUsize, Ordering};

const BUFFER_SIZE: usize = 1600;

/// Lock-free triple-buffer for audio oscilloscope capture.
///
/// Uses three buffers: one being written by the audio thread, one ready for
/// reading, and one in transition. This allows lock-free concurrent access
/// from the audio callback (writer) and UI thread (reader).
pub struct ScopeCapture {
    buffers: [Box<[f32; BUFFER_SIZE]>; 3],
    write_idx: AtomicUsize,
    write_buffer: AtomicUsize,
    read_buffer: AtomicUsize,
}

// SAFETY: All mutable access is through atomic operations or single-writer guarantee.
// The write methods are only called from one audio callback thread at a time.
unsafe impl Send for ScopeCapture {}
// SAFETY: Concurrent read/write is safe due to triple-buffering design.
// Writer and reader operate on different buffers, synchronized via atomics.
unsafe impl Sync for ScopeCapture {}

impl ScopeCapture {
    /// Creates a new scope capture with zeroed buffers.
    pub fn new() -> Self {
        Self {
            buffers: [
                Box::new([0.0; BUFFER_SIZE]),
                Box::new([0.0; BUFFER_SIZE]),
                Box::new([0.0; BUFFER_SIZE]),
            ],
            write_idx: AtomicUsize::new(0),
            write_buffer: AtomicUsize::new(0),
            read_buffer: AtomicUsize::new(2),
        }
    }

    /// Pushes a stereo sample pair, converting to mono for display.
    #[inline]
    pub fn push_stereo(&self, left: f32, right: f32) {
        let mono = (left + right) * 0.5;
        self.push_mono(mono);
    }

    /// Pushes a mono sample to the write buffer.
    #[inline]
    pub fn push_mono(&self, sample: f32) {
        let buf_idx = self.write_buffer.load(Ordering::Relaxed);
        let write_pos = self.write_idx.load(Ordering::Relaxed);

        let buf_ptr = self.buffers[buf_idx].as_ptr() as *mut f32;
        // SAFETY: write_pos is always < BUFFER_SIZE, and only one writer exists.
        unsafe {
            *buf_ptr.add(write_pos) = sample;
        }

        let next_pos = write_pos + 1;
        if next_pos >= BUFFER_SIZE {
            let next_buf = (buf_idx + 1) % 3;
            self.read_buffer.store(buf_idx, Ordering::Release);
            self.write_buffer.store(next_buf, Ordering::Relaxed);
            self.write_idx.store(0, Ordering::Relaxed);
        } else {
            self.write_idx.store(next_pos, Ordering::Relaxed);
        }
    }

    /// Returns peak (min, max) pairs for waveform display.
    pub fn read_peaks(&self, num_peaks: usize) -> Vec<(f32, f32)> {
        if num_peaks == 0 {
            return Vec::new();
        }

        let buf_idx = self.read_buffer.load(Ordering::Acquire);
        let buf = &self.buffers[buf_idx];

        let window = (BUFFER_SIZE / num_peaks).max(1);
        buf.chunks(window)
            .take(num_peaks)
            .map(|chunk| {
                chunk
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(min, max), &s| (min.min(s), max.max(s)))
            })
            .collect()
    }

    /// Returns a copy of the current read buffer samples.
    pub fn read_samples(&self) -> Vec<f32> {
        let buf_idx = self.read_buffer.load(Ordering::Acquire);
        self.buffers[buf_idx].to_vec()
    }

    /// Returns the buffer size in samples.
    pub const fn buffer_size() -> usize {
        BUFFER_SIZE
    }
}

impl Default for ScopeCapture {
    fn default() -> Self {
        Self::new()
    }
}
