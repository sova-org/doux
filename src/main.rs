//! Doux audio synthesis engine CLI.
//!
//! Provides real-time audio synthesis with OSC control. Supports sample
//! playback, multiple output channels, and live audio input processing.

use clap::Parser;
use cpal::traits::{DeviceTrait, StreamTrait};
use doux::audio::{
    default_input_device, default_output_device, find_input_device, find_output_device,
    list_input_devices, list_output_devices, max_output_channels,
};
use doux::Engine;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
}

fn print_devices() {
    println!("Input devices:");
    for info in list_input_devices() {
        let marker = if info.is_default { " *" } else { "" };
        println!("  {}: {}{}", info.index, info.name, marker);
    }

    println!("\nOutput devices:");
    for info in list_output_devices() {
        let marker = if info.is_default { " *" } else { "" };
        println!("  {}: {}{}", info.index, info.name, marker);
    }
}

fn main() {
    let args = Args::parse();

    if args.list_devices {
        print_devices();
        return;
    }

    // Resolve output device
    let device = match &args.output {
        Some(spec) => find_output_device(spec)
            .unwrap_or_else(|| panic!("output device '{spec}' not found")),
        None => default_output_device().expect("no output device"),
    };

    // Clamp channels to device maximum
    let max_channels = max_output_channels(&device);
    let output_channels = (args.channels as usize).min(max_channels as usize);
    if args.channels as usize > output_channels {
        eprintln!(
            "Warning: device supports max {} channels, using that instead of {}",
            max_channels, args.channels
        );
    }

    let default_config = device.default_output_config().unwrap();
    let sample_rate = default_config.sample_rate().0 as f32;

    let config = cpal::StreamConfig {
        channels: output_channels as u16,
        sample_rate: default_config.sample_rate(),
        buffer_size: args
            .buffer_size
            .map(cpal::BufferSize::Fixed)
            .unwrap_or(cpal::BufferSize::Default),
    };

    println!("Output: {}", device.name().unwrap_or_default());
    println!("Sample rate: {sample_rate}");
    println!("Channels: {output_channels}");
    if let Some(buf) = args.buffer_size {
        let latency_ms = buf as f32 / sample_rate * 1000.0;
        println!("Buffer: {buf} samples ({latency_ms:.1} ms)");
    }

    // Initialize engine with sample index if provided
    let mut engine = Engine::new_with_channels(sample_rate, output_channels, args.max_voices);

    if let Some(ref dir) = args.samples {
        println!("\nScanning samples from: {}", dir.display());
        let index = doux::loader::scan_samples_dir(dir);
        println!("Found {} samples (lazy loading enabled)\n", index.len());
        engine.sample_index = index;
    }

    let engine = Arc::new(Mutex::new(engine));

    // Ring buffer for live audio input
    let input_buffer: Arc<Mutex<VecDeque<f32>>> =
        Arc::new(Mutex::new(VecDeque::with_capacity(8192)));

    // Set up input stream if device available
    let input_device = match &args.input {
        Some(spec) => find_input_device(spec),
        None => default_input_device(),
    };
    let _input_stream = input_device.and_then(|input_device| {
        let input_config = input_device.default_input_config().ok()?;
        println!("Input: {}", input_device.name().unwrap_or_default());
        let buf = Arc::clone(&input_buffer);
        let stream = input_device
            .build_input_stream(
                &input_config.into(),
                move |data: &[f32], _| {
                    let mut b = buf.lock().unwrap();
                    for &sample in data {
                        b.push_back(sample);
                        if b.len() > 8192 {
                            b.pop_front();
                        }
                    }
                },
                |err| eprintln!("input stream error: {err}"),
                None,
            )
            .ok()?;
        stream.play().ok()?;
        Some(stream)
    });

    // Build output stream with audio callback
    let engine_clone = Arc::clone(&engine);
    let input_buf_clone = Arc::clone(&input_buffer);
    let live_scratch: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(vec![0.0; 1024]));
    let live_scratch_clone = Arc::clone(&live_scratch);
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                let mut buf = input_buf_clone.lock().unwrap();
                let mut scratch = live_scratch_clone.lock().unwrap();
                if scratch.len() < data.len() {
                    scratch.resize(data.len(), 0.0);
                }
                for sample in scratch[..data.len()].iter_mut() {
                    *sample = buf.pop_front().unwrap_or(0.0);
                }
                drop(buf);
                engine_clone
                    .lock()
                    .unwrap()
                    .process_block(data, &[], &scratch[..data.len()]);
            },
            |err| eprintln!("stream error: {err}"),
            None,
        )
        .unwrap();

    stream.play().unwrap();
    println!("Listening for OSC on port {}", args.port);
    println!("Press Ctrl+C to stop");

    // Block on OSC server (runs until interrupted)
    doux::osc::run(engine, args.port);
}
