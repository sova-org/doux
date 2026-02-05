//! Lock-free sample storage for real-time audio.
//!
//! Provides thread-safe sample access without mutex contention in the audio callback.
//! Uses atomic pointer swapping via `ArcSwap` for lock-free reads.

use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;

/// Immutable sample data that can be safely shared across threads.
///
/// Once created, sample data never changes, making it safe to share
/// via `Arc` without synchronization.
pub struct SampleData {
    /// Interleaved audio frames (immutable after creation).
    pub frames: Box<[f32]>,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u8,
    /// Base frequency in Hz for pitch calculations.
    pub freq: f32,
    /// Number of frames in the PCM buffer.
    pub frame_count: u32,
    /// Total frames in the original file (may differ from frame_count for head preloads).
    pub total_frames: u32,
}

impl SampleData {
    /// Creates new sample data from decoded audio.
    pub fn new(samples: Vec<f32>, channels: u8, freq: f32) -> Self {
        let frame_count = (samples.len() / channels as usize) as u32;
        Self {
            frames: samples.into_boxed_slice(),
            channels,
            freq,
            frame_count,
            total_frames: frame_count,
        }
    }

    /// Creates sample data for a head preload where total_frames may exceed frame_count.
    pub fn new_head(samples: Vec<f32>, channels: u8, freq: f32, total_frames: u32) -> Self {
        let frame_count = (samples.len() / channels as usize) as u32;
        Self {
            frames: samples.into_boxed_slice(),
            channels,
            freq,
            frame_count,
            total_frames,
        }
    }

    /// Reads a sample at the given frame and channel with linear interpolation.
    #[inline]
    pub fn read_interpolated(&self, pos: f32, channel: usize) -> f32 {
        let ch = channel.min(self.channels as usize - 1);
        let channels = self.channels as usize;

        let frame = pos as usize;
        let next_frame = (frame + 1).min(self.frame_count as usize - 1);
        let frac = pos.fract();

        let idx0 = frame * channels + ch;
        let idx1 = next_frame * channels + ch;

        let s0 = self.frames.get(idx0).copied().unwrap_or(0.0);
        let s1 = self.frames.get(idx1).copied().unwrap_or(0.0);

        s0 + frac * (s1 - s0)
    }
}

/// Lock-free registry for sample data.
///
/// Uses `ArcSwap` for atomic reads without blocking. Writers create a new
/// HashMap and atomically swap it in, while readers get a consistent snapshot.
pub struct SampleRegistry {
    samples: ArcSwap<HashMap<String, Arc<SampleData>>>,
}

impl Default for SampleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleRegistry {
    pub fn new() -> Self {
        Self {
            samples: ArcSwap::from_pointee(HashMap::new()),
        }
    }

    /// Gets a sample by name (lock-free).
    ///
    /// Returns a cloned `Arc` to the sample data, which can be held
    /// indefinitely without blocking other threads.
    #[inline]
    pub fn get(&self, name: &str) -> Option<Arc<SampleData>> {
        self.samples.load().get(name).cloned()
    }

    /// Inserts a sample into the registry (atomic swap).
    ///
    /// Creates a new HashMap with the sample added and atomically swaps it in.
    /// Existing readers continue using their snapshot until they reload.
    pub fn insert(&self, name: String, data: Arc<SampleData>) {
        let mut new_map = HashMap::clone(&self.samples.load());
        new_map.insert(name, data);
        self.samples.store(Arc::new(new_map));
    }

    /// Inserts many samples in a single atomic swap.
    pub fn insert_batch(&self, entries: impl IntoIterator<Item = (String, Arc<SampleData>)>) {
        let mut new_map = HashMap::clone(&self.samples.load());
        for (name, data) in entries {
            new_map.insert(name, data);
        }
        self.samples.store(Arc::new(new_map));
    }

    /// Checks if a sample exists (lock-free).
    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.samples.load().contains_key(name)
    }

    /// Returns the number of loaded samples.
    pub fn len(&self) -> usize {
        self.samples.load().len()
    }

    /// Returns true if no samples are loaded.
    pub fn is_empty(&self) -> bool {
        self.samples.load().is_empty()
    }
}
