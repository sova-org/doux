use std::path::PathBuf;
#[cfg(feature = "soundfont")]
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

#[cfg(feature = "soundfont")]
use crossbeam_channel::TrySendError;
use crossbeam_channel::{Receiver, Sender};

use doux::arc_swap::ArcSwap;
use doux::sampling::{scan_samples_dir, SampleEntry, SampleRegistry};
#[cfg(feature = "soundfont")]
use doux::telemetry::EngineMetrics;

use crate::manager::AudioCmd;

pub enum WorkerTask {
    RescanSamples(Vec<PathBuf>),
    AddSamplePath(PathBuf),
    #[cfg(feature = "soundfont")]
    LoadSoundfont(Vec<PathBuf>),
}

pub struct EngineWorker {
    pub tx: Sender<WorkerTask>,
    handle: JoinHandle<()>,
}

impl EngineWorker {
    /// `cmd_tx` is retained for the soundfont install path; sample-index
    /// updates go through `sample_index` directly and never round-trip
    /// through the audio thread.
    pub fn spawn(
        cmd_tx: Sender<AudioCmd>,
        registry: Arc<SampleRegistry>,
        sample_index: Arc<ArcSwap<Vec<SampleEntry>>>,
        sample_rate: f32,
        #[cfg(feature = "soundfont")] metrics: Arc<EngineMetrics>,
        #[cfg(not(feature = "soundfont"))] _metrics: Arc<doux::telemetry::EngineMetrics>,
    ) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<WorkerTask>();

        let handle = thread::Builder::new()
            .name("engine-worker".into())
            .spawn(move || {
                run(
                    rx,
                    cmd_tx,
                    registry,
                    sample_index,
                    sample_rate,
                    #[cfg(feature = "soundfont")]
                    metrics,
                );
            })
            .expect("failed to spawn engine worker thread");

        Self { tx, handle }
    }

    pub fn join(self) {
        drop(self.tx);
        let _ = self.handle.join();
    }
}

fn run(
    rx: Receiver<WorkerTask>,
    cmd_tx: Sender<AudioCmd>,
    registry: Arc<SampleRegistry>,
    sample_index: Arc<ArcSwap<Vec<SampleEntry>>>,
    sample_rate: f32,
    #[cfg(feature = "soundfont")] metrics: Arc<EngineMetrics>,
) {
    // `cmd_tx` is only consulted from the soundfont arm; bind silently when
    // the feature is off so clippy doesn't flag the unused parameter.
    #[cfg(not(feature = "soundfont"))]
    let _ = &cmd_tx;
    for task in &rx {
        match task {
            WorkerTask::RescanSamples(paths) => {
                let mut index = Vec::new();
                for path in &paths {
                    index.extend(scan_samples_dir(path));
                }
                spawn_preload(&index, sample_rate, &registry);
                // Atomic publish — audio thread sees the new index on its
                // next snapshot load. The old Vec is dropped here on the
                // worker thread.
                sample_index.store(Arc::new(index));
            }
            WorkerTask::AddSamplePath(path) => {
                let new_entries = scan_samples_dir(&path);
                spawn_preload(&new_entries, sample_rate, &registry);
                let mut next = (*sample_index.load_full()).clone();
                next.extend(new_entries);
                sample_index.store(Arc::new(next));
            }
            #[cfg(feature = "soundfont")]
            WorkerTask::LoadSoundfont(paths) => {
                for path in &paths {
                    if let Some(sf2_path) = doux::soundfont::find_sf2_file(path) {
                        match doux::soundfont::load_sf2(&sf2_path, sample_rate) {
                            Ok((samples, bank)) => {
                                let batch: Vec<_> = samples
                                    .into_iter()
                                    .map(|(name, data)| (name, Arc::new(data)))
                                    .collect();
                                match cmd_tx.try_send(AudioCmd::InstallSoundfont {
                                    bank,
                                    samples: batch,
                                }) {
                                    Ok(()) => {}
                                    Err(TrySendError::Full(_)) => {
                                        metrics.dropped_cmds.fetch_add(1, Ordering::Relaxed);
                                    }
                                    Err(TrySendError::Disconnected(_)) => {}
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "[doux] failed to load soundfont {}: {e}",
                                    sf2_path.display()
                                );
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
}

fn spawn_preload(index: &[SampleEntry], target_sr: f32, registry: &Arc<SampleRegistry>) {
    if index.is_empty() {
        return;
    }
    let entries: Vec<(Arc<str>, Arc<std::path::PathBuf>)> = index
        .iter()
        .map(|e| (e.name.clone(), e.path.clone()))
        .collect();
    let registry = Arc::clone(registry);
    std::thread::Builder::new()
        .name("sample-preload".into())
        .spawn(move || {
            let mut batch = Vec::with_capacity(entries.len());
            for (name, path) in &entries {
                match doux::sampling::decode_sample_head(path.as_ref(), target_sr) {
                    Ok(data) => batch.push((name.to_string(), Arc::new(data))),
                    Err(e) => eprintln!("[doux] preload {name}: {e}"),
                }
            }
            if !batch.is_empty() {
                registry.insert_batch(batch);
            }
        })
        .expect("failed to spawn preload thread");
}
