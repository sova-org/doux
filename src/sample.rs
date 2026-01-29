//! Sample storage and playback primitives.
//!
//! On native builds with lock-free registry, only [`SampleEntry`] is used for indexing.
//! On WASM builds, the legacy [`SamplePool`], [`SampleInfo`], and [`FileSource`] are used.

use std::path::PathBuf;

/// Index entry for a discoverable sample file.
///
/// Created during directory scanning with [`crate::loader::scan_samples_dir`].
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
    pub pos: f32,
    start_pos: f32,
    length: f32,
    started: bool,
}

#[cfg(not(feature = "native"))]
impl FileSource {
    pub fn new(sample_idx: usize, frames: u32, begin: f32, end: f32) -> Self {
        let begin = begin.clamp(0.0, 1.0);
        let end = end.clamp(0.0, 1.0);
        let (lo, hi) = if begin <= end { (begin, end) } else { (end, begin) };
        let fc = frames as f32;
        Self {
            sample_idx,
            pos: 0.0,
            start_pos: lo * fc,
            length: (hi - lo) * fc,
            started: false,
        }
    }

    #[inline]
    pub fn read(&self, pool: &[f32], channels: usize, offset: usize, channel: usize) -> f32 {
        let clamped = self.pos.clamp(0.0, (self.length - 1.0).max(0.0));
        let current = (self.start_pos + clamped) as usize;
        let frac = clamped.fract();
        let ch = channel.min(channels - 1);
        let end_frame = (self.start_pos + self.length) as usize;
        let idx0 = offset + current * channels + ch;
        let idx1 = if current + 1 < end_frame {
            offset + (current + 1) * channels + ch
        } else {
            idx0
        };
        let s0 = pool.get(idx0).copied().unwrap_or(0.0);
        let s1 = pool.get(idx1).copied().unwrap_or(0.0);
        s0 + frac * (s1 - s0)
    }

    #[inline]
    pub fn advance(&mut self, speed: f32) {
        if !self.started {
            self.started = true;
            if speed < 0.0 {
                self.pos = self.length;
            }
        }
        self.pos += speed;
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.pos < 0.0 || self.pos >= self.length
    }

    pub fn update_range(&mut self, frames: u32, begin: Option<f32>, end: Option<f32>) {
        let fc = frames as f32;
        let current_lo = self.start_pos / fc;
        let current_hi = current_lo + self.length / fc;
        let new_begin = begin.unwrap_or(current_lo).clamp(0.0, 1.0);
        let new_end = end.unwrap_or(current_hi).clamp(0.0, 1.0);
        let (lo, hi) = if new_begin <= new_end { (new_begin, new_end) } else { (new_end, new_begin) };
        self.start_pos = lo * fc;
        self.length = (hi - lo) * fc;
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
#[derive(Clone, Copy, Default)]
pub struct WebSampleSource {
    pub info: WebSampleInfo,
    pub pos: f32,
    start_pos: f32,
    length: f32,
    started: bool,
}

impl WebSampleSource {
    pub fn new(offset: usize, frames: u32, channels: u8, freq: f32, begin: f32, end: f32) -> Self {
        let begin = begin.clamp(0.0, 1.0);
        let end = end.clamp(0.0, 1.0);
        let fc = frames as f32;
        let (lo, hi) = if begin <= end { (begin, end) } else { (end, begin) };
        let length = (hi - lo) * fc;
        Self {
            info: WebSampleInfo {
                offset,
                channels,
                freq,
            },
            pos: 0.0,
            start_pos: lo * fc,
            length,
            started: false,
        }
    }

    #[inline]
    pub fn read(&self, pcm_buffer: &[f32], channel: usize) -> f32 {
        let clamped = self.pos.clamp(0.0, (self.length - 1.0).max(0.0));
        let current = (self.start_pos + clamped) as usize;
        let ch = channel.min(self.info.channels as usize - 1);
        let idx = self.info.offset + current * self.info.channels as usize + ch;
        pcm_buffer.get(idx).copied().unwrap_or(0.0)
    }

    #[inline]
    pub fn advance(&mut self, speed: f32) {
        if !self.started {
            self.started = true;
            if speed < 0.0 {
                self.pos = self.length;
            }
        }
        self.pos += speed;
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.pos < 0.0 || self.pos >= self.length
    }
}
