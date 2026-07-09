//! Audio recorder. RT-safe by construction.
//!
//! The audio thread only pushes captured interleaved-stereo samples into a
//! wait-free SPSC ring (`capture_block`); it never allocates, locks, or
//! blocks. A `RecorderWriter` thread drains the ring continuously and builds
//! the destination buffer off-RT, finalizing into the sample registry on stop.
//! Overdub mixing also runs on the writer — the RT side is identical for
//! record and overdub. Replaces the old toggle + per-stop 23 MB `Vec` alloc
//! that page-faulted on the audio thread.

use crate::sampling::{SampleData, SampleEntry, SampleRegistry};
use crate::telemetry::EngineMetrics;
use crate::types::{CHANNELS, MAX_BUFFER_FRAMES};
use arc_swap::ArcSwap;
use crossbeam_channel::{bounded, Receiver, RecvTimeoutError, Sender, TrySendError};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Ring capacity in stereo frames. ~1.36 s @ 48 k stereo (512 KB). The writer
/// drains every few ms, so this only absorbs scheduling jitter.
const RING_FRAMES: usize = 1 << 16;
/// Writer drain cadence — also the ctrl-channel poll timeout.
const WRITER_POLL: Duration = Duration::from_millis(2);
/// Soft per-recording safety cap (~10 min stereo @ 48 k). Replaces the old
/// hard 60 s cap; stops a forgotten recording from growing without bound.
const WRITER_MAX_SECONDS: usize = 600;
/// Ctrl queue depth. Start/stop are user-paced; 8 covers any burst.
const CTRL_DEPTH: usize = 8;

enum State {
    Idle,
    Active,
}

/// Writer control message. `name` is an `Arc<String>` so start carries no
/// extra allocation beyond the one the RT side already makes for the UI mirror.
enum RecCtrl {
    Start { name: Arc<String>, overdub: bool },
    Stop,
}

pub struct Recorder {
    state: State,
    target_orbit: Option<usize>,
    producer: HeapProd<f32>,
    writer: RecorderWriter,
    metrics: Arc<EngineMetrics>,
    // Reused so `stop()` publishes idle state without allocating.
    empty_name: Arc<String>,
    // Downmix staging for the non-stereo master path. Sized once; never grows.
    scratch: Vec<f32>,
    elapsed_frames: u64,
}

impl Recorder {
    pub fn new(
        sr: f32,
        metrics: Arc<EngineMetrics>,
        registry: Arc<SampleRegistry>,
        sample_index: Arc<ArcSwap<Vec<SampleEntry>>>,
    ) -> Self {
        let (mut producer, mut consumer) = HeapRb::<f32>::new(RING_FRAMES * CHANNELS).split();
        // Pre-fault the ring off-RT so first capture takes no page faults.
        prefault(&mut producer, &mut consumer);
        let writer = RecorderWriter::spawn(consumer, registry, sample_index, sr);
        Self {
            state: State::Idle,
            target_orbit: None,
            producer,
            writer,
            metrics,
            empty_name: Arc::new(String::new()),
            scratch: vec![0.0; MAX_BUFFER_FRAMES * CHANNELS],
            elapsed_frames: 0,
        }
    }

    pub fn target_orbit(&self) -> Option<usize> {
        self.target_orbit
    }

    /// RT-safe. Begins a recording; no-op if one is already active (illegal
    /// double-start ignored). The single `Arc::from(name)` is the only
    /// allocation — once per user action, never per block.
    pub fn start(&mut self, name: String, overdub: bool, target_orbit: Option<usize>) {
        if matches!(self.state, State::Active) {
            return;
        }
        let name = Arc::new(name);
        self.state = State::Active;
        self.target_orbit = target_orbit;
        self.elapsed_frames = 0;
        self.publish(true, overdub, target_orbit, Arc::clone(&name));
        let _ = self.writer.send(RecCtrl::Start { name, overdub });
    }

    /// RT-safe. Stops the active recording; no-op if idle.
    pub fn stop(&mut self) {
        if matches!(self.state, State::Idle) {
            return;
        }
        self.state = State::Idle;
        self.target_orbit = None;
        self.publish(false, false, None, Arc::clone(&self.empty_name));
        let _ = self.writer.send(RecCtrl::Stop);
    }

    // Mirror recording state for host UIs. Atomics + one small `Arc` store.
    fn publish(&self, active: bool, overdub: bool, orbit: Option<usize>, name: Arc<String>) {
        self.metrics.rec_active.store(active, Ordering::Relaxed);
        self.metrics.rec_overdub.store(overdub, Ordering::Relaxed);
        self.metrics
            .rec_orbit
            .store(orbit.map_or(u32::MAX, |o| o as u32), Ordering::Relaxed);
        self.metrics.rec_elapsed_frames.store(0, Ordering::Relaxed);
        self.metrics.rec_name.store(name);
    }

    /// RT capture. Pushes interleaved stereo into the ring. Zero alloc, no
    /// lock, no panic. Drops the tail (bumping `dropped_cmds`) only if the
    /// writer has fallen >1.3 s behind — never blocks.
    #[inline]
    pub fn capture_block(&mut self, output: &[f32], block_samples: usize, output_channels: usize) {
        if matches!(self.state, State::Idle) {
            return;
        }
        let want = block_samples * CHANNELS;
        let pushed = if output_channels == CHANNELS {
            self.producer.push_slice(&output[..want.min(output.len())])
        } else {
            // Downmix to stereo (front L/R; mono duplicates L) into scratch.
            let in_frames = output.len().checked_div(output_channels).unwrap_or(0);
            let frames = block_samples
                .min(self.scratch.len() / CHANNELS)
                .min(in_frames);
            let r_off = (output_channels >= 2) as usize;
            for i in 0..frames {
                let base = i * output_channels;
                self.scratch[i * CHANNELS] = output[base];
                self.scratch[i * CHANNELS + 1] = output[base + r_off];
            }
            self.producer.push_slice(&self.scratch[..frames * CHANNELS])
        };
        if pushed < want {
            self.metrics.dropped_cmds.fetch_add(1, Ordering::Relaxed);
        }
        self.elapsed_frames += (pushed / CHANNELS) as u64;
        self.metrics
            .rec_elapsed_frames
            .store(self.elapsed_frames, Ordering::Relaxed);
    }
}

// Touch every ring slot once so its pages are resident before RT use.
fn prefault(producer: &mut HeapProd<f32>, consumer: &mut HeapCons<f32>) {
    let zeros = [0.0f32; 4096];
    while producer.vacant_len() > 0 && producer.push_slice(&zeros) > 0 {}
    let mut sink = [0.0f32; 4096];
    while consumer.pop_slice(&mut sink) > 0 {}
}

/// Background thread: drains the capture ring, builds the recording off-RT,
/// and finalizes it into the registry on stop.
pub struct RecorderWriter {
    ctrl_tx: Option<Sender<RecCtrl>>,
    handle: Option<JoinHandle<()>>,
}

struct ActiveWrite {
    name: Arc<String>,
    overdub: bool,
    write_pos: usize,
}

impl RecorderWriter {
    fn spawn(
        consumer: HeapCons<f32>,
        registry: Arc<SampleRegistry>,
        sample_index: Arc<ArcSwap<Vec<SampleEntry>>>,
        sr: f32,
    ) -> Self {
        let (ctrl_tx, ctrl_rx) = bounded::<RecCtrl>(CTRL_DEPTH);
        let max_len = WRITER_MAX_SECONDS * sr as usize * CHANNELS;
        let handle = thread::Builder::new()
            .name("recorder-writer".into())
            .spawn(move || writer_loop(consumer, ctrl_rx, &registry, &sample_index, max_len))
            .expect("failed to spawn recorder writer thread");
        Self {
            ctrl_tx: Some(ctrl_tx),
            handle: Some(handle),
        }
    }

    // Non-blocking, RT-safe. Drops the message if the queue is full (should
    // never happen for user-paced start/stop).
    fn send(&self, ctrl: RecCtrl) -> Result<(), RecCtrl> {
        let Some(tx) = self.ctrl_tx.as_ref() else {
            return Err(ctrl);
        };
        match tx.try_send(ctrl) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(c)) | Err(TrySendError::Disconnected(c)) => Err(c),
        }
    }
}

impl Drop for RecorderWriter {
    fn drop(&mut self) {
        // Drop sender → writer loop sees Disconnected, finalizes, exits.
        self.ctrl_tx.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn writer_loop(
    mut consumer: HeapCons<f32>,
    ctrl_rx: Receiver<RecCtrl>,
    registry: &SampleRegistry,
    sample_index: &ArcSwap<Vec<SampleEntry>>,
    max_len: usize,
) {
    let mut dest: Vec<f32> = Vec::new();
    let mut active: Option<ActiveWrite> = None;
    let mut chunk = [0.0f32; 4096];
    loop {
        if let Some(a) = active.as_mut() {
            drain(&mut consumer, &mut chunk, &mut dest, a, max_len);
        }
        match ctrl_rx.recv_timeout(WRITER_POLL) {
            Ok(RecCtrl::Start { name, overdub }) => {
                // Defensive: finalize a take that never received Stop.
                if let Some(mut a) = active.take() {
                    drain(&mut consumer, &mut chunk, &mut dest, &mut a, max_len);
                    finalize(registry, sample_index, &a.name, std::mem::take(&mut dest));
                }
                dest.clear();
                let mut overdub = overdub;
                if overdub {
                    if let Some(data) = registry.get(&format!("{name}/0")) {
                        dest.extend_from_slice(&data.frames);
                    }
                    // Target missing → fall back to a fresh recording.
                    if dest.is_empty() {
                        overdub = false;
                    }
                }
                active = Some(ActiveWrite {
                    name,
                    overdub,
                    write_pos: 0,
                });
            }
            Ok(RecCtrl::Stop) => {
                if let Some(mut a) = active.take() {
                    drain(&mut consumer, &mut chunk, &mut dest, &mut a, max_len);
                    finalize(registry, sample_index, &a.name, std::mem::take(&mut dest));
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                if let Some(mut a) = active.take() {
                    drain(&mut consumer, &mut chunk, &mut dest, &mut a, max_len);
                    finalize(registry, sample_index, &a.name, std::mem::take(&mut dest));
                }
                break;
            }
        }
    }
}

// Drain the ring into `dest`: append for record, sum-with-wrap for overdub.
fn drain(
    consumer: &mut HeapCons<f32>,
    chunk: &mut [f32],
    dest: &mut Vec<f32>,
    active: &mut ActiveWrite,
    max_len: usize,
) {
    loop {
        let n = consumer.pop_slice(chunk);
        if n == 0 {
            break;
        }
        if active.overdub {
            overdub_mix(dest, &chunk[..n], &mut active.write_pos);
        } else if dest.len() < max_len {
            let room = max_len - dest.len();
            dest.extend_from_slice(&chunk[..n.min(room)]);
        }
    }
}

// Sum interleaved-stereo `src` onto `dest` starting at `write_pos`, wrapping
// at the end of the existing buffer (layering onto a fixed-length loop).
fn overdub_mix(dest: &mut [f32], src: &[f32], write_pos: &mut usize) {
    let buf_len = dest.len();
    if buf_len < CHANNELS {
        return;
    }
    let mut pos = *write_pos;
    let mut i = 0;
    while i + 1 < src.len() {
        if pos + 1 >= buf_len {
            pos = 0;
        }
        dest[pos] += src[i];
        dest[pos + 1] += src[i + 1];
        pos += CHANNELS;
        i += CHANNELS;
    }
    *write_pos = pos;
}

// Build the SampleData and publish it into the registry + index. Off-RT.
fn finalize(
    registry: &SampleRegistry,
    sample_index: &ArcSwap<Vec<SampleEntry>>,
    name: &str,
    captured: Vec<f32>,
) {
    if captured.is_empty() {
        return;
    }
    let data = SampleData::new(captured, CHANNELS as u8, 261.626);
    let key = format!("{name}/0");
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
