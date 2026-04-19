use std::path::PathBuf;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam_channel::{Receiver, Sender};

use doux::sampling::{scan_samples_dir, SampleEntry, SampleRegistry};

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
    pub fn spawn(
        cmd_tx: Sender<AudioCmd>,
        registry: Arc<SampleRegistry>,
        sample_rate: f32,
    ) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<WorkerTask>();

        let handle = thread::Builder::new()
            .name("engine-worker".into())
            .spawn(move || {
                run(rx, cmd_tx, registry, sample_rate);
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
    sample_rate: f32,
) {
    for task in &rx {
        match task {
            WorkerTask::RescanSamples(paths) => {
                let mut index = Vec::new();
                for path in &paths {
                    index.extend(scan_samples_dir(path));
                }
                spawn_preload(&index, sample_rate, &registry);
                let _ = cmd_tx.send(AudioCmd::SetSampleIndex(index));
            }
            WorkerTask::AddSamplePath(path) => {
                let index = scan_samples_dir(&path);
                spawn_preload(&index, sample_rate, &registry);
                let _ = cmd_tx.send(AudioCmd::ExtendSampleIndex(index));
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
                                let _ = cmd_tx.send(AudioCmd::InstallSoundfont {
                                    bank,
                                    samples: batch,
                                });
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
    let entries: Vec<(String, std::path::PathBuf)> = index
        .iter()
        .map(|e| (e.name.clone(), e.path.clone()))
        .collect();
    let registry = Arc::clone(registry);
    std::thread::Builder::new()
        .name("sample-preload".into())
        .spawn(move || {
            let mut batch = Vec::with_capacity(entries.len());
            for (name, path) in &entries {
                match doux::sampling::decode_sample_head(path, target_sr) {
                    Ok(data) => batch.push((name.clone(), Arc::new(data))),
                    Err(e) => eprintln!("[doux] preload {name}: {e}"),
                }
            }
            if !batch.is_empty() {
                registry.insert_batch(batch);
            }
        })
        .expect("failed to spawn preload thread");
}
