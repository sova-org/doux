//! Doux audio synthesis engine CLI.
//!
//! Provides real-time audio synthesis with OSC control. Supports sample
//! playback, multiple output channels, and live audio input processing.

use clap::Parser;
use doux::audio::{get_host, print_diagnostics, HostSelection};
use doux::cli_common::{
    build_audio_streams, print_devices, print_hosts, recreate_engine, resolve_output_config,
    StreamParams,
};
use doux::AudioCmd;
use doux::Engine;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Command-line arguments for the doux audio engine.
#[derive(Parser)]
#[command(name = "doux")]
#[command(about = "Audio synthesis engine with OSC control", long_about = None)]
struct Args {
    /// Directory containing audio samples to load.
    #[arg(short, long)]
    samples: Option<PathBuf>,

    /// OSC port to listen on.
    #[arg(short, long, default_value = "57120")]
    port: u16,

    /// List available audio devices and exit.
    #[arg(long)]
    list_devices: bool,

    /// Input device (name or index).
    #[arg(short, long)]
    input: Option<String>,

    /// Output device (name or index).
    #[arg(short, long)]
    output: Option<String>,

    /// Number of output channels (default: 2, max depends on device).
    #[arg(long, default_value = "2")]
    channels: u16,

    /// Audio buffer size in samples (lower = less latency, higher = more stable).
    /// Common values: 64, 128, 256, 512, 1024. Default: system choice.
    #[arg(short, long)]
    buffer_size: Option<u32>,

    /// Maximum polyphony (number of simultaneous voices).
    #[arg(long, default_value = "32")]
    max_voices: usize,

    /// Preload all samples at startup (blocks until complete).
    #[arg(long)]
    preload: bool,

    /// Audio host backend: jack, alsa, or auto (default: auto).
    /// On Linux with PipeWire, use 'jack' for best compatibility.
    #[arg(long, default_value = "auto")]
    host: String,

    /// Run audio diagnostics and exit (useful for troubleshooting on Linux).
    #[arg(long)]
    diagnose: bool,
}

fn main() {
    let args = Args::parse();

    let host_selection: HostSelection = args.host.parse().unwrap_or_else(|e| panic!("{e}"));

    if args.diagnose {
        print_hosts();
        println!();
        print_diagnostics();
        return;
    }

    let host = get_host(host_selection).unwrap_or_else(|e| panic!("{e}"));

    if args.list_devices {
        print_devices(&host);
        return;
    }

    let oc = resolve_output_config(
        &host,
        args.output.as_deref(),
        args.channels,
        args.buffer_size,
    );

    println!("Audio host: {}", host.id().name());
    if let Some(buf) = args.buffer_size {
        let latency_ms = buf as f32 / oc.sample_rate * 1000.0;
        println!("Buffer: {buf} samples ({latency_ms:.1} ms)");
    }

    let block_size = args
        .buffer_size
        .map(|b| b as usize)
        .unwrap_or(doux::types::DEFAULT_NATIVE_BLOCK_SIZE);
    let mut engine = Engine::new_with_channels(
        oc.sample_rate,
        oc.output_channels,
        args.max_voices,
        block_size,
    );

    if let Some(ref dir) = args.samples {
        println!("\nScanning samples from: {}", dir.display());
        let index = doux::sampling::scan_samples_dir(dir);
        let sample_count = index.len();

        if args.preload {
            println!("Preloading {sample_count} samples...");
            for entry in &index {
                match doux::sampling::decode_sample_file(&entry.path, oc.sample_rate) {
                    Ok(data) => {
                        engine
                            .sample_registry
                            .insert(entry.name.clone(), Arc::new(data));
                    }
                    Err(e) => {
                        eprintln!("Failed to preload {}: {e}", entry.name);
                    }
                }
            }
            println!("Preloaded {} samples\n", engine.sample_registry.len());
        } else {
            println!("Found {sample_count} samples (lazy loading enabled)\n");
        }

        engine.sample_index = index;

        #[cfg(feature = "soundfont")]
        engine.load_soundfont_from_dir(dir);
    }

    let sample_index = engine.sample_index.clone();
    let sample_registry = Arc::clone(&engine.sample_registry);
    #[cfg(feature = "soundfont")]
    let gm_bank = engine.gm_bank.take();

    let device_lost = Arc::new(AtomicBool::new(false));

    println!("Listening for OSC on port {}", args.port);
    println!("Press Ctrl+C to stop");

    let (mut cmd_tx, mut cmd_rx) = crossbeam_channel::unbounded::<AudioCmd>();

    let stream_params = StreamParams {
        host: &host,
        input_spec: args.input.as_deref(),
        output_spec: args.output.as_deref(),
        config: &oc,
        device_lost: &device_lost,
    };

    loop {
        let streams = build_audio_streams(&stream_params, engine, cmd_rx);

        let lost = doux::osc::run_recoverable(cmd_tx.clone(), args.port, &device_lost);

        drop(streams);

        if !lost {
            break;
        }

        eprintln!("Audio device lost, attempting to reconnect...");
        device_lost.store(false, Ordering::Release);
        std::thread::sleep(std::time::Duration::from_secs(1));

        engine = recreate_engine(
            oc.sample_rate,
            oc.output_channels,
            args.max_voices,
            block_size,
            &sample_index,
            &sample_registry,
            #[cfg(feature = "soundfont")]
            &gm_bank,
        );
        let (new_tx, new_rx) = crossbeam_channel::unbounded::<AudioCmd>();
        cmd_tx = new_tx;
        cmd_rx = new_rx;
    }
}
