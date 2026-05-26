//! Per-orbit / per-output audio recorder with off-RT finalize.
//!
//! The audio thread only writes captured samples into a pre-allocated `Vec`
//! via `capture_block`. Toggling recording off via `toggle_rt` moves the
//! captured buffer out by `mem::take` and exposes it for hand-off to a
//! worker thread that builds the `SampleData`, inserts it into the registry,
//! and updates the sample index.
//!
//! Allocations on the audio thread are bounded to a single
//! `Vec::with_capacity(max_len)` per recording-stop (refilling `active` for
//! the next take). The previously RT-hot work — `Arc::new(SampleData)`,
//! `SampleRegistry::insert` (HashMap clone), `ArcSwap<Vec<SampleEntry>>`
//! update (Vec clone) — now runs on the worker.

use crate::arc_swap::ArcSwap;
use crate::sampling::{SampleData, SampleEntry, SampleRegistry};
use crate::types::CHANNELS;
use crossbeam_channel::{bounded, Sender, TrySendError};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

const MAX_RECORD_SECONDS: usize = 60;
/// Worker queue depth. Recording is a user-initiated action; back-to-back
/// finalize-faster-than-publish is rare. 4 covers any plausible burst.
const FINALIZE_QUEUE_DEPTH: usize = 4;

enum State {
    Idle,
    Recording,
    Overdubbing,
}

/// Result of a `toggle_rt` call.
pub enum RecorderRtResult {
    /// Toggle had no effect (no active recording to finalize), or recording
    /// just started.
    None,
    /// Recording stopped. Captured buffer and chosen name are returned for
    /// hand-off to the off-RT worker.
    Finalized { name: String, captured: Vec<f32> },
}

/// Off-RT finalize job.
pub struct RecorderJob {
    pub name: String,
    pub captured: Vec<f32>,
}

pub struct Recorder {
    active: Vec<f32>,
    write_pos: usize,
    max_len: usize,
    state: State,
    name: String,
    counter: usize,
    target_orbit: Option<usize>,
}

impl Recorder {
    pub fn new(sr: f32) -> Self {
        let max_len = MAX_RECORD_SECONDS * sr as usize * CHANNELS;
        Self {
            active: Vec::with_capacity(max_len),
            write_pos: 0,
            max_len,
            state: State::Idle,
            name: String::new(),
            counter: 0,
            target_orbit: None,
        }
    }

    pub fn target_orbit(&self) -> Option<usize> {
        self.target_orbit
    }

    /// RT-safe toggle. Returns the captured buffer on stop so the caller can
    /// hand it to the worker thread.
    ///
    /// Allocation profile on the audio thread:
    /// - Start: zero-to-one `String` alloc (`format!("rec{N}")` only if `name`
    ///   is `None`); one `format!("{name}/0")` + `extend_from_slice` (capacity
    ///   pre-allocated) only if `overdub`.
    /// - Stop: `Vec::with_capacity(max_len)` to refill `active`. The captured
    ///   `Vec<f32>` is moved out by `mem::take`, not copied.
    pub fn toggle_rt(
        &mut self,
        name: Option<String>,
        overdub: bool,
        target_orbit: Option<usize>,
        registry: &SampleRegistry,
    ) -> RecorderRtResult {
        match self.state {
            State::Idle => {
                let rec_name = name.unwrap_or_else(|| {
                    let n = format!("rec{}", self.counter);
                    self.counter += 1;
                    n
                });

                if overdub {
                    self.active.clear();
                    let key = format!("{rec_name}/0");
                    if let Some(data) = registry.get(&key) {
                        let src = &data.frames;
                        if src.len() <= self.max_len {
                            self.active.extend_from_slice(src);
                        }
                    }
                    self.write_pos = 0;
                    self.state = State::Overdubbing;
                } else {
                    self.active.clear();
                    self.state = State::Recording;
                }

                self.name = rec_name;
                self.target_orbit = target_orbit;
                RecorderRtResult::None
            }
            State::Recording | State::Overdubbing => {
                self.state = State::Idle;
                self.target_orbit = None;
                if self.active.is_empty() {
                    return RecorderRtResult::None;
                }
                let captured = std::mem::take(&mut self.active);
                // Re-prime `active` for the next take. One Vec::with_capacity
                // allocation per recording-stop. The expensive work
                // (SampleData::new, registry.insert, sample_index update)
                // now runs on the worker — see [`RecorderWorker`].
                self.active = Vec::with_capacity(self.max_len);
                let name = std::mem::take(&mut self.name);
                RecorderRtResult::Finalized { name, captured }
            }
        }
    }

    #[inline]
    pub fn capture_block(&mut self, output: &[f32], block_samples: usize, output_channels: usize) {
        match self.state {
            State::Idle => {}
            State::Recording => self.record_block(output, block_samples, output_channels),
            State::Overdubbing => self.overdub_block(output, block_samples, output_channels),
        }
    }

    #[inline]
    fn record_block(&mut self, output: &[f32], block_samples: usize, output_channels: usize) {
        let remaining = self.max_len - self.active.len();
        if remaining == 0 {
            return;
        }

        if output_channels == CHANNELS {
            let n = (block_samples * CHANNELS).min(remaining);
            self.active.extend_from_slice(&output[..n]);
        } else {
            let max_frames = remaining / CHANNELS;
            let frames = block_samples.min(max_frames);
            for i in 0..frames {
                let base = i * output_channels;
                self.active.push(output[base]);
                self.active.push(output[base + 1]);
            }
        }
    }

    #[inline]
    fn overdub_block(&mut self, output: &[f32], block_samples: usize, output_channels: usize) {
        let buf_len = self.active.len();
        if buf_len == 0 {
            self.record_block(output, block_samples, output_channels);
            return;
        }

        for i in 0..block_samples {
            let base = i * output_channels;
            let l = output[base];
            let r = output[base + 1];

            if self.write_pos >= buf_len {
                self.write_pos = 0;
            }

            self.active[self.write_pos] += l;
            self.active[self.write_pos + 1] += r;
            self.write_pos += CHANNELS;
        }
    }
}

/// Background thread that turns captured recordings into registry entries.
///
/// Receives [`RecorderJob`]s from the audio thread via a bounded channel and
/// performs the formerly RT-unsafe work (Box<[f32]> alloc, HashMap clone,
/// Vec<SampleEntry> clone, ArcSwap store) off the audio thread.
pub struct RecorderWorker {
    tx: Option<Sender<RecorderJob>>,
    handle: Option<JoinHandle<()>>,
}

impl RecorderWorker {
    pub fn spawn(
        registry: Arc<SampleRegistry>,
        sample_index: Arc<ArcSwap<Vec<SampleEntry>>>,
    ) -> Self {
        let (tx, rx) = bounded::<RecorderJob>(FINALIZE_QUEUE_DEPTH);
        let handle = thread::Builder::new()
            .name("recorder-finalize".into())
            .spawn(move || {
                for job in rx {
                    let data = SampleData::new(job.captured, CHANNELS as u8, 261.626);
                    let key = format!("{}/0", job.name);
                    registry.insert(key.clone(), Arc::new(data));
                    let current = sample_index.load_full();
                    if !current.iter().any(|e| e.name.as_ref() == key) {
                        let mut new_index = (*current).clone();
                        new_index.push(SampleEntry {
                            name: Arc::from(key),
                            path: Arc::new(PathBuf::new()),
                        });
                        sample_index.store(Arc::new(new_index));
                    }
                }
            })
            .expect("failed to spawn recorder worker thread");
        Self {
            tx: Some(tx),
            handle: Some(handle),
        }
    }

    /// Non-blocking send. On full or disconnected, drops the job.
    pub fn try_send(&self, job: RecorderJob) -> Result<(), RecorderJob> {
        let Some(tx) = self.tx.as_ref() else {
            return Err(job);
        };
        match tx.try_send(job) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(j)) | Err(TrySendError::Disconnected(j)) => Err(j),
        }
    }
}

impl Drop for RecorderWorker {
    fn drop(&mut self) {
        // Drop sender to close the channel; the worker's `for job in rx` loop
        // exits cleanly. Join to make destruction deterministic.
        self.tx.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
