//! Shared native offline engine runner for rendering and benchmarking.

use crate::sampling::{decode_sample_file, scan_samples_dir};
use crate::Engine;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OfflineEngineConfig {
    pub sample_rate: f32,
    pub channels: usize,
    pub max_voices: usize,
    pub block_size: usize,
}

impl Default for OfflineEngineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000.0,
            channels: 2,
            max_voices: 64,
            block_size: 512,
        }
    }
}

#[derive(Debug, Default)]
pub struct OfflinePass {
    pub elapsed_ns: u64,
    pub samples: usize,
    pub blocks: usize,
    pub output: Option<Vec<f32>>,
}

pub fn create_engine(
    config: OfflineEngineConfig,
    samples_dir: Option<&Path>,
) -> Result<Engine, String> {
    let mut engine = Engine::new_with_channels(
        config.sample_rate,
        config.channels,
        config.max_voices,
        config.block_size,
    );

    if let Some(dir) = samples_dir {
        let index = scan_samples_dir(dir);
        for entry in &index {
            let data = decode_sample_file(&entry.path, config.sample_rate)
                .map_err(|err| format!("failed to load {}: {err}", entry.name))?;
            engine
                .sample_registry
                .insert(entry.name.clone(), Arc::new(data));
        }
        engine.sample_index = index;

        #[cfg(feature = "soundfont")]
        engine.load_soundfont_from_dir(dir);
    }

    Ok(engine)
}

pub fn apply_setup_commands<I, S>(engine: &mut Engine, commands: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    for command in commands {
        engine.evaluate(command.as_ref());
    }
}

pub fn render_to_buffer(
    engine: &mut Engine,
    duration_seconds: f32,
) -> OfflinePass {
    run_engine(engine, duration_seconds, true)
}

pub fn run_without_capture(
    engine: &mut Engine,
    duration_seconds: f32,
) -> OfflinePass {
    run_engine(engine, duration_seconds, false)
}

fn run_engine(engine: &mut Engine, duration_seconds: f32, capture_output: bool) -> OfflinePass {
    let total_samples = seconds_to_samples(engine.sr, duration_seconds);
    let channels = engine.output_channels;
    let block_samples = engine.block_size.max(1);
    let mut rendered_samples = 0usize;
    let mut blocks = 0usize;
    let mut scratch = vec![0.0f32; block_samples * channels];
    let mut output = capture_output.then(|| vec![0.0f32; total_samples * channels]);

    let start = Instant::now();
    while rendered_samples < total_samples {
        let chunk_samples = (total_samples - rendered_samples).min(block_samples);
        let chunk_len = chunk_samples * channels;
        let chunk = if let Some(buffer) = output.as_mut() {
            &mut buffer[rendered_samples * channels..rendered_samples * channels + chunk_len]
        } else {
            &mut scratch[..chunk_len]
        };

        engine.process_block(chunk, &[], &[]);
        rendered_samples += chunk_samples;
        blocks += 1;
    }

    OfflinePass {
        elapsed_ns: start.elapsed().as_nanos() as u64,
        samples: total_samples,
        blocks,
        output,
    }
}

pub fn seconds_to_samples(sample_rate: f32, duration_seconds: f32) -> usize {
    if duration_seconds <= 0.0 {
        0
    } else {
        (sample_rate * duration_seconds) as usize
    }
}
