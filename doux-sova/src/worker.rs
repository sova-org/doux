use std::path::PathBuf;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam_channel::{Receiver, Sender};

use doux::arc_swap::ArcSwap;
#[cfg(feature = "soundfont")]
use doux::arc_swap::ArcSwapOption;
use doux::sampling::{scan_samples_dir, SampleEntry, SampleIndex, SampleRegistry};
#[cfg(feature = "soundfont")]
use doux::soundfont::GmBank;

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
    /// Sample-index and soundfont updates are published directly off-RT (via
    /// the `sample_index` / `gm_bank` ArcSwaps); nothing round-trips through the
    /// audio thread.
    pub fn spawn(
        registry: Arc<SampleRegistry>,
        sample_index: Arc<ArcSwap<SampleIndex>>,
        sample_rate: f32,
        #[cfg(feature = "soundfont")] gm_bank: Arc<ArcSwapOption<GmBank>>,
    ) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<WorkerTask>();

        let handle = thread::Builder::new()
            .name("engine-worker".into())
            .spawn(move || {
                run(
                    rx,
                    registry,
                    sample_index,
                    sample_rate,
                    #[cfg(feature = "soundfont")]
                    gm_bank,
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
    registry: Arc<SampleRegistry>,
    sample_index: Arc<ArcSwap<SampleIndex>>,
    sample_rate: f32,
    #[cfg(feature = "soundfont")] gm_bank: Arc<ArcSwapOption<GmBank>>,
) {
    for task in &rx {
        match task {
            WorkerTask::RescanSamples(paths) => {
                let mut index = Vec::new();
                for path in &paths {
                    index.extend(scan_samples_dir(path));
                }
                spawn_preload(&index, sample_rate, &registry);
                // Atomic publish — audio thread sees the new index on its
                // next snapshot load. Building the `SampleIndex` sorts here,
                // on the worker thread, so lookup stays a binary search. The
                // old index is dropped here too.
                sample_index.store(Arc::new(SampleIndex::new(index)));
            }
            WorkerTask::AddSamplePath(path) => {
                let new_entries = scan_samples_dir(&path);
                spawn_preload(&new_entries, sample_rate, &registry);
                let mut next = sample_index.load().entries().to_vec();
                next.extend(new_entries);
                sample_index.store(Arc::new(SampleIndex::new(next)));
            }
            #[cfg(feature = "soundfont")]
            WorkerTask::LoadSoundfont(paths) => {
                for path in &paths {
                    if let Some(sf2_path) = doux::soundfont::find_sf2_file(path) {
                        match doux::soundfont::load_sf2(&sf2_path, sample_rate) {
                            Ok(bank) => {
                                // The bank owns its sample PCM, so publishing is a
                                // single atomic store off the RT thread — a GM note
                                // never resolves a zone whose sample isn't present.
                                // The old bank drops here on the worker.
                                gm_bank.store(Some(Arc::new(bank)));
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
