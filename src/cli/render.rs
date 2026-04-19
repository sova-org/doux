//! Offline WAV rendering for doux.
//!
//! Renders audio synthesis to a WAV file instead of real-time playback.

use clap::Parser;
use doux::offline::{apply_setup_commands, create_engine, render_to_buffer, OfflineEngineConfig};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::PathBuf;

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

    let config = OfflineEngineConfig {
        sample_rate: args.sample_rate as f32,
        channels: args.channels as usize,
        max_voices: args.max_voices,
        block_size: 512,
    };

    let mut engine =
        create_engine(config, args.samples.as_deref()).unwrap_or_else(|err| panic!("{err}"));
    apply_setup_commands(&mut engine, &args.eval);
    let output = render_to_buffer(&mut engine, args.duration)
        .output
        .expect("offline render should capture output");

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
