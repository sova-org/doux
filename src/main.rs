//! Doux audio synthesis engine CLI.
//!
//! Provides real-time audio synthesis with OSC control. Supports sample
//! playback, multiple output channels, and live audio input processing.

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use doux::audio::{get_host, list_hosts, max_output_channels, print_diagnostics, HostSelection};
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

fn print_devices(host: &cpal::Host) {
    let default_in = host.default_input_device().and_then(|d| d.name().ok());
    let default_out = host.default_output_device().and_then(|d| d.name().ok());

    println!("Audio host: {}", host.id().name());

    println!("\nInput devices:");
    if let Ok(devices) = host.input_devices() {
        for (i, d) in devices.enumerate() {
            let name = d.name().unwrap_or_else(|_| "???".into());
            let marker = if Some(&name) == default_in.as_ref() {
                " *"
            } else {
                ""
            };
            println!("  {i}: {name}{marker}");
        }
    } else {
        println!("  (no input devices available)");
    }

    println!("\nOutput devices:");
    if let Ok(devices) = host.output_devices() {
        for (i, d) in devices.enumerate() {
            let name = d.name().unwrap_or_else(|_| "???".into());
            let marker = if Some(&name) == default_out.as_ref() {
                " *"
            } else {
                ""
            };
            println!("  {i}: {name}{marker}");
        }
    } else {
        println!("  (no output devices available)");
    }
}

fn print_hosts() {
    println!("Available audio hosts:");
    for h in list_hosts() {
        let status = if h.available { "" } else { " (unavailable)" };
        println!("  {}{}", h.name, status);
    }
}

fn find_device<I>(devices: I, spec: &str) -> Option<cpal::Device>
where
    I: Iterator<Item = cpal::Device>,
{
    let devices: Vec<_> = devices.collect();
    if let Ok(idx) = spec.parse::<usize>() {
        return devices.into_iter().nth(idx);
    }
    let spec_lower = spec.to_lowercase();
    devices.into_iter().find(|d| {
        d.name()
            .map(|n| n.to_lowercase().contains(&spec_lower))
            .unwrap_or(false)
    })
}

fn main() {
    let args = Args::parse();

    // Parse host selection
    let host_selection: HostSelection = args.host.parse().unwrap_or_else(|e| panic!("{e}"));

    // Handle diagnose flag first
    if args.diagnose {
        print_hosts();
        println!();
        print_diagnostics();
        return;
    }

    // Get the audio host
    let host = get_host(host_selection).unwrap_or_else(|e| panic!("{e}"));

    if args.list_devices {
        print_devices(&host);
        return;
    }

    // Resolve output device
    let device = match &args.output {
        Some(spec) => host
            .output_devices()
            .ok()
            .and_then(|d| find_device(d, spec))
            .unwrap_or_else(|| panic!("output device '{spec}' not found")),
        None => host.default_output_device().expect("no output device"),
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

    println!("Audio host: {}", host.id().name());
    println!(
        "Output: {} @ {}Hz, {}ch",
        device.name().unwrap_or_default(),
        sample_rate as u32,
        output_channels
    );
    if let Some(buf) = args.buffer_size {
        let latency_ms = buf as f32 / sample_rate * 1000.0;
        println!("Buffer: {buf} samples ({latency_ms:.1} ms)");
    }

    // Initialize engine with sample index if provided
    let mut engine = Engine::new_with_channels(sample_rate, output_channels, args.max_voices);

    if let Some(ref dir) = args.samples {
        println!("\nScanning samples from: {}", dir.display());
        let index = doux::sampling::scan_samples_dir(dir);
        let sample_count = index.len();

        if args.preload {
            println!("Preloading {sample_count} samples...");
            for entry in &index {
                match doux::sampling::decode_sample_file(&entry.path, sample_rate) {
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
    }

    let engine = Arc::new(Mutex::new(engine));

    // Ring buffer for live audio input
    let input_buffer: Arc<Mutex<VecDeque<f32>>> =
        Arc::new(Mutex::new(VecDeque::with_capacity(8192)));

    // Set up input stream if device available
    let input_device = match &args.input {
        Some(spec) => host.input_devices().ok().and_then(|d| find_device(d, spec)),
        None => host.default_input_device(),
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
