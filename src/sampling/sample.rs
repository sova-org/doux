//! Sample storage and playback primitives.
//!
//! On native builds with lock-free registry, only [`SampleEntry`] is used for indexing.
//! On WASM builds, the legacy [`SamplePool`], [`SampleInfo`], and [`FileSource`] are used.

use std::path::PathBuf;

use super::cursor::Cursor;

/// Index entry for a discoverable sample file.
///
/// Created during directory scanning with [`super::scan_samples_dir`].
pub struct SampleEntry {
    /// Filesystem path to the audio file.
    pub path: PathBuf,
    /// Display name (derived from filename or folder/index).
    pub name: String,
}

/// Contiguous storage for all loaded sample data (WASM only).
#[cfg(not(feature = "native"))]
#[derive(Default)]
pub struct SamplePool {
    pub data: Vec<f32>,
}

#[cfg(not(feature = "native"))]
impl SamplePool {
    pub fn new() -> Self {
        Self::default()
    }

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
}

/// Metadata for a sample stored in the pool (WASM only).
#[cfg(not(feature = "native"))]
#[derive(Clone, Copy, Default)]
pub struct SampleInfo {
    pub offset: usize,
    pub frames: u32,
    pub channels: u8,
    pub freq: f32,
}

/// Playback cursor for reading samples from the pool (WASM only).
#[cfg(not(feature = "native"))]
#[derive(Clone, Copy)]
pub struct FileSource {
    pub sample_idx: usize,
    cursor: Cursor,
}

#[cfg(not(feature = "native"))]
impl FileSource {
    pub fn new(sample_idx: usize, frames: u32, begin: f32, end: f32) -> Self {
        Self {
            sample_idx,
            cursor: Cursor::new(frames, begin, end),
        }
    }

    /// Reads the sample value at current position with linear interpolation.
    #[inline]
    pub fn read(&self, pool: &[f32], channels: usize, offset: usize, channel: usize) -> f32 {
        let ch = channel.min(channels - 1);
        let current = self.cursor.current_frame();
        let frac = self.cursor.frac();

        let idx0 = offset + current * channels + ch;
        let idx1 = offset + self.cursor.next_frame(u32::MAX) * channels + ch;

        let s0 = pool.get(idx0).copied().unwrap_or(0.0);
        let s1 = pool.get(idx1).copied().unwrap_or(0.0);
        s0 + frac * (s1 - s0)
    }

    #[inline]
    pub fn advance(&mut self, speed: f32) {
        self.cursor.advance(speed);
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.cursor.is_done()
    }

    pub fn update_range(&mut self, frames: u32, begin: Option<f32>, end: Option<f32>) {
        self.cursor.update_range(frames, begin, end);
    }
}

/// Sample info for WebSampleSource (used on all platforms for web sample playback).
#[derive(Clone, Copy, Default)]
pub struct WebSampleInfo {
    pub offset: usize,
    pub channels: u8,
    pub freq: f32,
}

/// Simplified sample playback for WASM environments.
#[derive(Clone, Copy)]
pub struct WebSampleSource {
    pub info: WebSampleInfo,
    cursor: Cursor,
}

impl Default for WebSampleSource {
    fn default() -> Self {
        Self {
            info: WebSampleInfo::default(),
            cursor: Cursor::default(),
        }
    }
}

impl WebSampleSource {
    pub fn new(offset: usize, frames: u32, channels: u8, freq: f32, begin: f32, end: f32) -> Self {
        Self {
            info: WebSampleInfo {
                offset,
                channels,
                freq,
            },
            cursor: Cursor::new(frames, begin, end),
        }
    }

    /// Reads the sample value at current position (no interpolation for web samples).
    #[inline]
    pub fn read(&self, pcm_buffer: &[f32], channel: usize) -> f32 {
        let ch = channel.min(self.info.channels as usize - 1);
        let current = self.cursor.current_frame();
        let idx = self.info.offset + current * self.info.channels as usize + ch;
        pcm_buffer.get(idx).copied().unwrap_or(0.0)
    }

    #[inline]
    pub fn advance(&mut self, speed: f32) {
        self.cursor.advance(speed);
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.cursor.is_done()
    }

    /// Returns the total frame count.
    #[inline]
    pub fn frame_count(&self) -> f32 {
        self.cursor.length()
    }
}
