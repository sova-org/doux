//! Sample storage and playback primitives.
//!
//! Provides a memory pool for audio samples and playback cursors for reading
//! them back at variable speeds with interpolation.
//!
//! # Architecture
//!
//! ```text
//! SampleEntry (index)     SamplePool (storage)      FileSource (playhead)
//! ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
//! │ path: kick.wav  │     │ [f32; N]         │     │ sample_idx: 0   │
//! │ name: "kick"    │────▶│ ├─ sample 0 ─────│◀────│ pos: 0.0        │
//! │ loaded: Some(0) │     │ ├─ sample 1      │     │ begin: 0.0      │
//! └─────────────────┘     │ └─ ...           │     │ end: 1.0        │
//!                         └──────────────────┘     └─────────────────┘
//! ```
//!
//! - [`SampleEntry`]: Metadata for lazy-loaded samples (path, name, pool index)
//! - [`SamplePool`]: Contiguous f32 storage for all loaded sample data
//! - [`SampleInfo`]: Location and format of a sample within the pool
//! - [`FileSource`]: Playback cursor with position, speed, and loop points
//! - [`WebSampleSource`]: Simplified playback for WASM (no interpolation)

use std::path::PathBuf;

/// Index entry for a discoverable sample file.
///
/// Created during directory scanning with [`crate::loader::scan_samples_dir`].
/// The `loaded` field is `None` until the sample is actually decoded and
/// added to the pool.
pub struct SampleEntry {
    /// Filesystem path to the audio file.
    pub path: PathBuf,
    /// Display name (derived from filename or folder/index).
    pub name: String,
    /// Pool index if loaded, `None` if not yet decoded.
    pub loaded: Option<usize>,
}

/// Contiguous storage for all loaded sample data.
///
/// Samples are stored sequentially as interleaved f32 frames. Each sample's
/// location is tracked by a corresponding [`SampleInfo`].
///
/// This design minimizes allocations and improves cache locality compared
/// to storing each sample in a separate `Vec`.
#[derive(Default)]
pub struct SamplePool {
    /// Raw interleaved sample data for all loaded samples.
    pub data: Vec<f32>,
}

impl SamplePool {
    /// Creates an empty sample pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds sample data to the pool and returns its metadata.
    ///
    /// The samples should be interleaved if multi-channel (e.g., `[L, R, L, R, ...]`).
    ///
    /// # Parameters
    ///
    /// - `samples`: Interleaved audio data
    /// - `channels`: Number of channels (1 = mono, 2 = stereo)
    /// - `freq`: Base frequency in Hz for pitch calculations
    ///
    /// # Returns
    ///
    /// [`SampleInfo`] describing the sample's location in the pool.
    pub fn add(&mut self, samples: &[f32], channels: u8, freq: f32) -> Option<SampleInfo> {
        let frames = samples.len() / channels as usize;
        let offset = self.data.len();

        let info = SampleInfo {
            offset,
            frames: frames as u32,
            channels,
            freq,
        };

        self.data.extend_from_slice(samples);
        Some(info)
    }

    /// Returns the total memory usage in megabytes.
    pub fn size_mb(&self) -> f32 {
        (self.data.len() * 4) as f32 / (1024.0 * 1024.0)
    }
}

/// Metadata for a sample stored in the pool.
///
/// Describes where a sample lives in the pool's data array and its format.
#[derive(Clone, Copy, Default)]
pub struct SampleInfo {
    /// Byte offset into [`SamplePool::data`] where this sample begins.
    pub offset: usize,
    /// Total number of frames (samples per channel).
    pub frames: u32,
    /// Number of interleaved channels.
    pub channels: u8,
    /// Base frequency in Hz (used for pitch-shifting calculations).
    pub freq: f32,
}

/// Playback cursor for reading samples from the pool.
///
/// Tracks playback position and supports:
/// - Variable-speed playback (including reverse with negative speed)
/// - Start/end points for partial playback or slicing
/// - Linear interpolation between samples for smooth pitch shifting
#[derive(Clone, Copy)]
pub struct FileSource {
    /// Index into the sample info array.
    pub sample_idx: usize,
    /// Current playback position in frames (fractional for interpolation).
    pub pos: f32,
    /// Start point as fraction of total length `[0.0, 1.0]`.
    pub begin: f32,
    /// End point as fraction of total length `[0.0, 1.0]`.
    pub end: f32,
}

impl Default for FileSource {
    fn default() -> Self {
        Self {
            sample_idx: 0,
            pos: 0.0,
            begin: 0.0,
            end: 1.0,
        }
    }
}

impl FileSource {
    /// Creates a new playback cursor for a sample with start/end points.
    ///
    /// Points are clamped to valid ranges: begin to `[0, 1]`, end to `[begin, 1]`.
    pub fn new(sample_idx: usize, begin: f32, end: f32) -> Self {
        let begin_clamped = begin.clamp(0.0, 1.0);
        Self {
            sample_idx,
            pos: 0.0,
            begin: begin_clamped,
            end: end.clamp(begin_clamped, 1.0),
        }
    }

    /// Reads the interpolated sample value at the current position.
    pub fn read(&self, pool: &[f32], info: &SampleInfo, channel: usize) -> f32 {
        let begin_frame = (self.begin * info.frames as f32) as usize;
        let end_frame = (self.end * info.frames as f32) as usize;
        let channels = info.channels as usize;

        let current = self.pos as usize + begin_frame;
        if current >= end_frame {
            return 0.0;
        }

        let frac = self.pos.fract();
        let ch = channel.min(channels - 1);

        let idx0 = info.offset + current * channels + ch;
        let idx1 = if current + 1 < end_frame {
            info.offset + (current + 1) * channels + ch
        } else {
            idx0
        };

        let s0 = pool.get(idx0).copied().unwrap_or(0.0);
        let s1 = pool.get(idx1).copied().unwrap_or(0.0);

        s0 + frac * (s1 - s0)
    }

    /// Advances the playback position by the given speed.
    pub fn advance(&mut self, speed: f32) {
        self.pos += speed;
    }

    /// Returns `true` if playback has reached or passed the end point.
    pub fn is_done(&self, info: &SampleInfo) -> bool {
        let begin_frame = (self.begin * info.frames as f32) as usize;
        let end_frame = (self.end * info.frames as f32) as usize;
        let current = self.pos as usize + begin_frame;
        current >= end_frame
    }
}

/// Simplified sample playback for WASM environments.
///
/// Unlike [`FileSource`], this struct embeds its [`SampleInfo`] and does not
/// perform interpolation. Designed for web playback where JavaScript populates
/// a shared PCM buffer that Rust reads from.
#[derive(Clone, Copy, Default)]
pub struct WebSampleSource {
    /// Sample metadata (location, size, format).
    pub info: SampleInfo,
    /// Current playback position in frames (relative to begin point).
    pub pos: f32,
    /// Normalized start point (0.0 = sample start, 1.0 = sample end).
    pub begin: f32,
    /// Normalized end point (0.0 = sample start, 1.0 = sample end).
    pub end: f32,
}

impl WebSampleSource {
    /// Creates a new sample source with the given loop points.
    ///
    /// Both `begin` and `end` are normalized values in the range 0.0 to 1.0,
    /// representing positions within the sample. Values are clamped automatically.
    pub fn new(info: SampleInfo, begin: f32, end: f32) -> Self {
        let begin_clamped = begin.clamp(0.0, 1.0);
        Self {
            info,
            pos: 0.0,
            begin: begin_clamped,
            end: end.clamp(begin_clamped, 1.0),
        }
    }

    /// Reads the sample value at the current position for the given channel.
    pub fn read(&self, pcm_buffer: &[f32], channel: usize) -> f32 {
        let begin_frame = (self.begin * self.info.frames as f32) as usize;
        let end_frame = (self.end * self.info.frames as f32) as usize;
        let current = self.pos as usize + begin_frame;

        if current >= end_frame {
            return 0.0;
        }

        let ch = channel.min(self.info.channels as usize - 1);
        let idx = self.info.offset + current * self.info.channels as usize + ch;
        pcm_buffer.get(idx).copied().unwrap_or(0.0)
    }

    /// Advances the playback position by the given speed.
    pub fn advance(&mut self, speed: f32) {
        self.pos += speed;
    }

    /// Returns true if playback has reached or passed the end point.
    pub fn is_done(&self) -> bool {
        let begin_frame = (self.begin * self.info.frames as f32) as usize;
        let end_frame = (self.end * self.info.frames as f32) as usize;
        let current = self.pos as usize + begin_frame;
        current >= end_frame
    }
}
