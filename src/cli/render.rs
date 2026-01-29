//! Offline WAV rendering for doux.
//!
//! Renders audio synthesis to a WAV file instead of real-time playback.

use clap::Parser;
use doux::sampling::{decode_sample_file, scan_samples_dir};
use doux::types::BLOCK_SIZE;
use doux::Engine;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "doux-render")]
#[command(about = "Render audio synthesis to WAV file", long_about = None)]
struct Args {
    /// Duration to render in seconds.
    #[arg(short, long)]
    duration: f32,

    /// Command to evaluate (can be repeated).
    #[arg(short, long)]
    eval: Vec<String>,

    /// Output WAV file path.
    #[arg(short, long)]
    output: PathBuf,

    /// Directory containing audio samples.
    #[arg(short, long)]
    samples: Option<PathBuf>,

    /// Sample rate (default: 48000).
    #[arg(long, default_value = "48000")]
    sample_rate: u32,

    /// Number of output channels (default: 2).
    #[arg(long, default_value = "2")]
    channels: u16,

    /// Maximum polyphony (default: 64).
    #[arg(long, default_value = "64")]
    max_voices: usize,
}

fn main() {
    let args = Args::parse();

    let sr = args.sample_rate as f32;
    let channels = args.channels as usize;

    let mut engine = Engine::new_with_channels(sr, channels, args.max_voices);

    if let Some(ref dir) = args.samples {
        let index = scan_samples_dir(dir);
        for entry in &index {
            match decode_sample_file(&entry.path, sr) {
                Ok(data) => {
                    engine
                        .sample_registry
                        .insert(entry.name.clone(), Arc::new(data));
                }
                Err(e) => {
                    eprintln!("Failed to load {}: {e}", entry.name);
                }
            }
        }
        engine.sample_index = index;
    }

    for cmd in &args.eval {
        engine.evaluate(cmd);
    }

    let total_samples = (sr * args.duration) as usize;
    let mut output = vec![0.0f32; total_samples * channels];

    for chunk in output.chunks_mut(BLOCK_SIZE * channels) {
        engine.process_block(chunk, &[], &[]);
    }

    let spec = WavSpec {
        channels: args.channels,
        sample_rate: args.sample_rate,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };

    let mut writer = WavWriter::create(&args.output, spec).expect("failed to create WAV file");
    for sample in output {
        writer.write_sample(sample).expect("failed to write sample");
    }
    writer.finalize().expect("failed to finalize WAV");

    println!(
        "Rendered {:.2}s to {} ({} Hz, {} ch)",
        args.duration,
        args.output.display(),
        args.sample_rate,
        args.channels
    );
}
