//! Background sample loading thread.
//!
//! Decouples sample loading from the audio/OSC threads to prevent blocking.
//! Uses a dedicated thread with crossbeam channel for lock-free communication.

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::loader::decode_sample_file;
use crate::sample_registry::SampleRegistry;

/// Request to load a sample from disk.
pub struct LoadRequest {
    /// Unique name for the sample (e.g., "kick/0").
    pub name: String,
    /// Path to the audio file.
    pub path: PathBuf,
    /// Target sample rate for resampling.
    pub target_sr: f32,
}

/// Background sample loader with dedicated thread.
pub struct SampleLoader {
    tx: Option<Sender<LoadRequest>>,
    handle: Option<JoinHandle<()>>,
}

impl SampleLoader {
    /// Creates a new background loader with the given registry.
    ///
    /// The loader thread will insert decoded samples directly into the registry.
    pub fn new(registry: Arc<SampleRegistry>) -> Self {
        let (tx, rx) = bounded::<LoadRequest>(64);

        let handle = thread::Builder::new()
            .name("sample-loader".into())
            .spawn(move || {
                loader_thread(rx, registry);
            })
            .expect("failed to spawn sample loader thread");

        Self {
            tx: Some(tx),
            handle: Some(handle),
        }
    }

    /// Requests a sample to be loaded in the background.
    ///
    /// Returns `true` if the request was queued, `false` if the queue is full.
    /// Non-blocking: will not wait if the channel is at capacity.
    pub fn request(&self, name: String, path: PathBuf, target_sr: f32) -> bool {
        let Some(ref tx) = self.tx else {
            return false;
        };
        match tx.try_send(LoadRequest {
            name,
            path,
            target_sr,
        }) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                eprintln!("Sample loader queue full, skipping request");
                false
            }
            Err(TrySendError::Disconnected(_)) => false,
        }
    }
}

impl Drop for SampleLoader {
    fn drop(&mut self) {
        // Close channel first by dropping sender
        self.tx.take();
        // Now join the thread - it will exit when it sees channel closed
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn loader_thread(rx: Receiver<LoadRequest>, registry: Arc<SampleRegistry>) {
    let mut pending: HashSet<String> = HashSet::new();

    for request in rx {
        if registry.contains(&request.name) || pending.contains(&request.name) {
            continue;
        }

        pending.insert(request.name.clone());

        match decode_sample_file(&request.path, request.target_sr) {
            Ok(data) => {
                registry.insert(request.name.clone(), Arc::new(data));
            }
            Err(e) => {
                eprintln!("Failed to load sample {}: {e}", request.name);
            }
        }

        pending.remove(&request.name);
    }
}
