//! Sample storage and playback primitives.
//!
//! On native builds with lock-free registry, only [`SampleEntry`] is used for indexing.
//! On WASM builds, the legacy [`SamplePool`], [`SampleInfo`], and [`FileSource`] are used.

use std::path::PathBuf;
use std::sync::Arc;

use super::cursor::Cursor;

/// Index entry for a discoverable sample file.
///
/// Created during directory scanning with [`super::scan_samples_dir`].
#[derive(Clone)]
pub struct SampleEntry {
    /// Filesystem path to the audio file.
    pub path: Arc<PathBuf>,
    /// Display name (derived from filename or folder/index).
    pub name: Arc<str>,
}

impl SampleEntry {
    /// True when this entry is `<folder>/<n>`. The one definition of what it
    /// means for a bare sound name to name a sample folder.
    pub fn in_folder(&self, folder: &str) -> bool {
        self.name.len() > folder.len()
            && self.name.as_bytes()[folder.len()] == b'/'
            && self.name.as_bytes().starts_with(folder.as_bytes())
    }

    /// Split `<folder>/<n>` into its parts. `None` for a top-level entry
    /// (a bare stem, no `/`) or a suffix that is not a plain integer.
    #[cfg(feature = "native")]
    fn folder_parts(&self) -> Option<(&str, usize)> {
        let (folder, suffix) = self.name.split_once('/')?;
        Some((folder, suffix.parse().ok()?))
    }
}

/// One folder's contiguous run inside [`SampleIndex::entries`].
#[cfg(feature = "native")]
struct FolderRange {
    name: Arc<str>,
    start: u32,
    /// Members are dense `0..count`, established by [`SampleIndex::new`].
    count: u32,
}

/// The sample index, arranged for lookup instead of for scanning.
///
/// `folder/n` resolution happens on the audio thread — event dispatch drains
/// inside the cpal callback — so the cost must not scale with the size of the
/// user's sample library. [`SampleIndex::new`] pays one sort off the RT thread
/// and every later lookup is a binary search over folder names plus a direct
/// index.
#[cfg(feature = "native")]
pub struct SampleIndex {
    entries: Vec<SampleEntry>,
    /// Sorted by `name`, one per distinct folder. Top-level entries have no
    /// range: they are addressed by exact name through the registry, never as
    /// `folder/n`.
    folders: Vec<FolderRange>,
}

#[cfg(feature = "native")]
impl SampleIndex {
    /// Group `entries` by folder and record each run.
    ///
    /// Sorting by `(folder, n)` rather than merely grouping is what lets
    /// `folder_entry` index directly: it makes the "one contiguous, densely
    /// numbered run per folder" invariant hold no matter what order the
    /// entries arrived in, so an append from `extend_sample_index` or the
    /// recorder cannot break the lookup.
    pub fn new(mut entries: Vec<SampleEntry>) -> Self {
        entries.sort_by(|a, b| match (a.folder_parts(), b.folder_parts()) {
            (Some((fa, na)), Some((fb, nb))) => fa.cmp(fb).then(na.cmp(&nb)),
            // Top-level entries sort after all folder members, keeping every
            // folder run contiguous. Their relative order is by name.
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        });
        // Two scanned roots that share a folder name both number their members
        // from 0, so the index can carry the same `folder/n` twice. The
        // registry is keyed by that name and cannot hold both, so the later
        // one is unreachable regardless — dropping it here is what keeps each
        // run densely numbered. The sort is stable, so the survivor is the
        // first-scanned entry, which is the one the old linear `find` picked.
        entries.dedup_by(|a, b| a.name == b.name);

        let mut folders: Vec<FolderRange> = Vec::new();
        for (i, entry) in entries.iter().enumerate() {
            let Some((folder, _)) = entry.folder_parts() else {
                break; // top-level tail; nothing past here belongs to a folder
            };
            match folders.last_mut() {
                Some(last) if last.name.as_ref() == folder => last.count += 1,
                _ => folders.push(FolderRange {
                    name: Arc::from(folder),
                    start: i as u32,
                    count: 1,
                }),
            }
        }

        Self { entries, folders }
    }

    /// The entry for `folder/n`, wrapping `n` modulo the folder's size.
    /// `None` when no such folder exists.
    pub fn folder_entry(&self, folder: &str, n: usize) -> Option<&SampleEntry> {
        let i = self
            .folders
            .binary_search_by(|r| r.name.as_ref().cmp(folder))
            .ok()?;
        let range = &self.folders[i];
        let wrapped = n % range.count as usize;
        let entry = &self.entries[range.start as usize + wrapped];
        debug_assert_eq!(
            entry.folder_parts(),
            Some((folder, wrapped)),
            "folder run for `{folder}` is not densely numbered from 0"
        );
        Some(entry)
    }

    /// True when `folder` names a sample folder.
    pub fn has_folder(&self, folder: &str) -> bool {
        self.folders
            .binary_search_by(|r| r.name.as_ref().cmp(folder))
            .is_ok()
    }

    pub fn entries(&self) -> &[SampleEntry] {
        &self.entries
    }
}

#[cfg(feature = "native")]
impl Default for SampleIndex {
    fn default() -> Self {
        Self::new(Vec::new())
    }
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
        let channels = channels.max(1);
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
#[derive(Clone, Copy, Default)]
pub struct WebSampleSource {
    pub info: WebSampleInfo,
    cursor: Cursor,
}

impl WebSampleSource {
    pub fn new(offset: usize, frames: u32, channels: u8, freq: f32, begin: f32, end: f32) -> Self {
        let channels = channels.max(1);
        let freq = if freq.is_finite() && freq > 0.0 {
            freq
        } else {
            261.626
        };
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
