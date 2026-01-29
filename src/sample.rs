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
    pub begin: f32,
    pub end: f32,
}

#[cfg(not(feature = "native"))]
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

#[cfg(not(feature = "native"))]
impl FileSource {
    pub fn new(sample_idx: usize, begin: f32, end: f32) -> Self {
        let begin_clamped = begin.clamp(0.0, 1.0);
        Self {
            sample_idx,
            pos: 0.0,
            begin: begin_clamped,
            end: end.clamp(begin_clamped, 1.0),
        }
    }

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

    pub fn advance(&mut self, speed: f32) {
        self.pos += speed;
    }

    pub fn is_done(&self, info: &SampleInfo) -> bool {
        let begin_frame = (self.begin * info.frames as f32) as usize;
        let end_frame = (self.end * info.frames as f32) as usize;
        let current = self.pos as usize + begin_frame;
        current >= end_frame
    }
}

/// Sample info for WebSampleSource (used on all platforms for web sample playback).
#[derive(Clone, Copy, Default)]
pub struct WebSampleInfo {
    pub offset: usize,
    pub frames: u32,
    pub channels: u8,
    pub freq: f32,
}

/// Simplified sample playback for WASM environments.
#[derive(Clone, Copy, Default)]
pub struct WebSampleSource {
    pub info: WebSampleInfo,
    pub pos: f32,
    pub begin: f32,
    pub end: f32,
}

impl WebSampleSource {
    pub fn new(offset: usize, frames: u32, channels: u8, freq: f32, begin: f32, end: f32) -> Self {
        let begin_clamped = begin.clamp(0.0, 1.0);
        Self {
            info: WebSampleInfo {
                offset,
                frames,
                channels,
                freq,
            },
            pos: 0.0,
            begin: begin_clamped,
            end: end.clamp(begin_clamped, 1.0),
        }
    }

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

    pub fn advance(&mut self, speed: f32) {
        self.pos += speed;
    }

    pub fn is_done(&self) -> bool {
        let begin_frame = (self.begin * self.info.frames as f32) as usize;
        let end_frame = (self.end * self.info.frames as f32) as usize;
        let current = self.pos as usize + begin_frame;
        current >= end_frame
    }
}
