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
    /// Wavetable cycle length the file declares, in source-file samples.
    /// `0` when the file is not a wavetable.
    pub wt_cycle_len: u32,
    /// Buffer frames per source-file frame (`target_sr / source_sr`), `1.0` when
    /// the file was not resampled. Cycle lengths are quoted in source-file
    /// samples, so they scale by this. Not to be confused with
    /// `RegistrySample::sr_ratio`, which is the inverse and feeds playback speed.
    pub resample_ratio: f32,
}

impl SampleData {
    /// Creates new sample data from decoded audio.
    pub fn new(samples: Vec<f32>, channels: u8, freq: f32) -> Self {
        let channels = channels.max(1);
        let frame_count = (samples.len() / channels as usize) as u32;
        Self {
            frames: samples.into_boxed_slice(),
            channels,
            freq,
            frame_count,
            total_frames: frame_count,
            wt_cycle_len: 0,
            resample_ratio: 1.0,
        }
    }

    /// Creates sample data for a head preload where total_frames may exceed frame_count.
    pub fn new_head(samples: Vec<f32>, channels: u8, freq: f32, total_frames: u32) -> Self {
        let channels = channels.max(1);
        let frame_count = (samples.len() / channels as usize) as u32;
        Self {
            frames: samples.into_boxed_slice(),
            channels,
            freq,
            frame_count,
            total_frames,
            wt_cycle_len: 0,
            resample_ratio: 1.0,
        }
    }

    /// Tags decoded data with what the file says about its wavetable layout.
    pub fn with_wavetable(mut self, wt_cycle_len: u32, resample_ratio: f32) -> Self {
        self.wt_cycle_len = wt_cycle_len;
        self.resample_ratio = resample_ratio;
        self
    }

    /// Cycle length in stored-buffer frames, resampling accounted for.
    ///
    /// `override_len` is the caller's cycle length in source-file samples, `0`
    /// to use whatever the file declared. Falls back to the whole buffer as a
    /// single cycle, which is what a one-cycle waveform file wants.
    #[inline]
    pub fn cycle_frames(&self, override_len: u32) -> f32 {
        let source_len = if override_len > 0 {
            override_len
        } else {
            self.wt_cycle_len
        };
        if source_len == 0 {
            return self.frame_count as f32;
        }
        source_len as f32 * self.resample_ratio
    }

    /// The four Hermite taps around `frame` on channel `ch`, clamped to the buffer's ends.
    ///
    /// The interior path — the whole window inside the buffer — takes one bounds-checked
    /// sub-slice covering all four taps and indexes within it, so the three remaining reads
    /// need no check of their own. It is bit-identical to the edge path: away from the ends
    /// every clamp is a no-op, so both read the same four elements.
    #[inline]
    fn taps(&self, frame: usize, last: usize, ch: usize) -> (f32, f32, f32, f32) {
        let channels = self.channels as usize;
        let frames = &self.frames;
        if frame >= 1 && frame + 2 <= last {
            let base = (frame - 1) * channels + ch;
            let w = &frames[base..base + 3 * channels + 1];
            (w[0], w[channels], w[2 * channels], w[3 * channels])
        } else {
            let i0 = frame.saturating_sub(1);
            let i2 = (frame + 1).min(last);
            let i3 = (frame + 2).min(last);
            (
                frames[i0 * channels + ch],
                frames[frame * channels + ch],
                frames[i2 * channels + ch],
                frames[i3 * channels + ch],
            )
        }
    }

    /// Reads a sample at the given frame and channel with 4-tap cubic Hermite interpolation.
    #[inline]
    pub fn read_interpolated(&self, pos: f32, channel: usize) -> f32 {
        let frame_count = self.frame_count as usize;
        if frame_count == 0 {
            return 0.0;
        }
        let ch = channel.min(self.channels as usize - 1);
        let last = frame_count - 1;

        let frame = (pos as usize).min(last);
        let frac = pos.fract();

        let (y0, y1, y2, y3) = self.taps(frame, last, ch);
        crate::dsp::hermite4(y0, y1, y2, y3, frac)
    }

    /// Reads output channels 0 and 1 at one position, sharing the frame, fraction and tap
    /// indices between them.
    ///
    /// Equal by construction to `read_interpolated(pos, 0)` and `read_interpolated(pos, 1)`:
    /// mono data clamps both channels onto channel 0, so one Hermite evaluation feeds both.
    #[inline]
    pub fn read_interpolated_stereo(&self, pos: f32) -> [f32; 2] {
        let frame_count = self.frame_count as usize;
        if frame_count == 0 {
            return [0.0; 2];
        }
        let last = frame_count - 1;

        let frame = (pos as usize).min(last);
        let frac = pos.fract();

        let (y0, y1, y2, y3) = self.taps(frame, last, 0);
        let left = crate::dsp::hermite4(y0, y1, y2, y3, frac);
        if self.channels < 2 {
            return [left, left];
        }
        let (z0, z1, z2, z3) = self.taps(frame, last, 1);
        [left, crate::dsp::hermite4(z0, z1, z2, z3, frac)]
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
        // `rcu` retries the clone-modify on a concurrent write, so multiple
        // off-RT writers (loader, recorder, soundfont worker) can't lose updates.
        // The closure may run more than once, so clone inside it.
        self.samples.rcu(|cur| {
            let mut new_map = HashMap::clone(cur);
            new_map.insert(name.clone(), Arc::clone(&data));
            Arc::new(new_map)
        });
    }

    /// Inserts many samples in a single atomic swap.
    pub fn insert_batch(&self, entries: impl IntoIterator<Item = (String, Arc<SampleData>)>) {
        // Collect once: `rcu`'s closure may retry, but an iterator is single-use.
        let entries: Vec<(String, Arc<SampleData>)> = entries.into_iter().collect();
        self.samples.rcu(|cur| {
            let mut new_map = HashMap::clone(cur);
            for (name, data) in &entries {
                new_map.insert(name.clone(), Arc::clone(data));
            }
            Arc::new(new_map)
        });
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
