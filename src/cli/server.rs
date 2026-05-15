//! Doux audio synthesis engine CLI.
//!
//! Provides real-time audio synthesis with OSC control. Supports sample
//! playback, multiple output channels, and live audio input processing.

use clap::Parser;
use doux::cli_common::{
    build_audio_streams, init_audio_host, recreate_engine, setup_engine_samples, CommonAudioArgs,
    HostInit, StreamParams,
};
use doux::AudioCmd;
use doux::Engine;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Command-line arguments for the doux audio engine.
#[derive(Parser)]
#[command(name = "doux")]
#[command(about = "Audio synthesis engine with OSC control", long_about = None)]
struct Args {
    #[command(flatten)]
    common: CommonAudioArgs,

    /// OSC port to listen on.
    #[arg(short, long, default_value = "57120")]
    port: u16,

    /// Preload all samples at startup (blocks until complete).
    #[arg(long)]
    preload: bool,
}

fn main() {
    let args = Args::parse();

    let (host, oc, block_size) = match init_audio_host(&args.common) {
        Ok(HostInit::Ready {
            host,
            output_config,
            block_size,
        }) => (host, output_config, block_size),
        Ok(HostInit::EarlyExit) => return,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    println!("Audio host: {}", host.id().name());
    if let Some(buf) = args.common.buffer_size {
        let latency_ms = buf as f32 / oc.sample_rate * 1000.0;
        println!("Buffer: {buf} samples ({latency_ms:.1} ms)");
    }

    let mut engine = Engine::new_with_channels(
        oc.sample_rate,
        oc.output_channels,
        args.common.max_voices,
        block_size,
    );

    if let Some(ref dir) = args.common.samples {
        setup_engine_samples(&mut engine, dir, args.preload, true);
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
        input_spec: args.common.input.as_deref(),
        output_spec: args.common.output.as_deref(),
        config: &oc,
        device_lost: &device_lost,
    };

    loop {
        let anchor = engine.time_anchor();
        let streams = match build_audio_streams(&stream_params, engine, cmd_rx) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        };

        let lost = match doux::osc::run_recoverable(cmd_tx.clone(), args.port, anchor, &device_lost)
        {
            Ok(lost) => lost,
            Err(e) => {
                eprintln!("Error binding OSC port {}: {e}", args.port);
                std::process::exit(1);
            }
        };

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
            args.common.max_voices,
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
