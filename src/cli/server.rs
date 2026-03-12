//! Doux audio synthesis engine CLI.
//!
//! Provides real-time audio synthesis with OSC control. Supports sample
//! playback, multiple output channels, and live audio input processing.

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::Receiver;
use doux::audio::{get_host, list_hosts, max_output_channels, print_diagnostics, HostSelection};
use doux::osc::AudioCmd;
use doux::Engine;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
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

fn print_devices(host: &cpal::Host) {
    let default_in = host
        .default_input_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));
    let default_out = host
        .default_output_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));

    println!("Audio host: {}", host.id().name());

    println!("\nInput devices:");
    if let Ok(devices) = host.input_devices() {
        for (i, d) in devices.enumerate() {
            let name = d
                .description()
                .map(|desc| desc.name().to_string())
                .unwrap_or_else(|_| "???".into());
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
            let name = d
                .description()
                .map(|desc| desc.name().to_string())
                .unwrap_or_else(|_| "???".into());
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
        d.description()
            .map(|desc| desc.name().to_lowercase().contains(&spec_lower))
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

    // Resolve output device for initial config (sample rate, channels)
    let (output_channels, sample_rate, config) = {
        let device = match &args.output {
            Some(spec) => host
                .output_devices()
                .ok()
                .and_then(|d| find_device(d, spec))
                .unwrap_or_else(|| panic!("output device '{spec}' not found")),
            None => host.default_output_device().expect("no output device"),
        };

        let max_channels = max_output_channels(&device);
        let output_channels = (args.channels as usize).min(max_channels as usize);
        if args.channels as usize > output_channels {
            eprintln!(
                "Warning: device supports max {} channels, using that instead of {}",
                max_channels, args.channels
            );
        }

        let default_config = device.default_output_config().unwrap();
        let sample_rate = default_config.sample_rate() as f32;

        let is_jack = doux::audio::is_jack_host();
        let buffer_size = match args.buffer_size {
            Some(buf) if !is_jack => cpal::BufferSize::Fixed(buf),
            Some(_) => {
                eprintln!("Note: JACK controls buffer size, ignoring -b flag");
                cpal::BufferSize::Default
            }
            None => cpal::BufferSize::Default,
        };

        let config = cpal::StreamConfig {
            channels: output_channels as u16,
            sample_rate: default_config.sample_rate(),
            buffer_size,
        };

        (output_channels, sample_rate, config)
    };

    println!("Audio host: {}", host.id().name());
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

    loop {
        let streams = build_streams(
            &host,
            &args,
            &config,
            output_channels,
            sample_rate,
            engine,
            cmd_rx,
            &device_lost,
        );

        let lost = doux::osc::run_recoverable(cmd_tx.clone(), args.port, &device_lost);

        drop(streams);

        if !lost {
            break;
        }

        eprintln!("Audio device lost, attempting to reconnect...");
        device_lost.store(false, Ordering::Release);
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Recreate engine and channel for reconnect
        engine = Engine::new_with_channels(sample_rate, output_channels, args.max_voices);
        engine.sample_index = sample_index.clone();
        engine.sample_registry = Arc::clone(&sample_registry);
        #[cfg(feature = "soundfont")]
        {
            engine.gm_bank = gm_bank.clone();
        }
        let (new_tx, new_rx) = crossbeam_channel::unbounded::<AudioCmd>();
        cmd_tx = new_tx;
        cmd_rx = new_rx;
    }
}

struct Streams {
    _output: cpal::Stream,
    _input: Option<cpal::Stream>,
}

fn build_streams(
    host: &cpal::Host,
    args: &Args,
    config: &cpal::StreamConfig,
    output_channels: usize,
    sample_rate: f32,
    mut engine: Engine,
    cmd_rx: Receiver<AudioCmd>,
    device_lost: &Arc<AtomicBool>,
) -> Streams {
    let input_device = match &args.input {
        Some(spec) => host.input_devices().ok().and_then(|d| find_device(d, spec)),
        None => doux::audio::default_input_device(),
    };

    let input_channels: usize = input_device
        .as_ref()
        .and_then(|dev| dev.default_input_config().ok())
        .map_or(0, |cfg| cfg.channels() as usize);

    let input_buffer_size = 8192 * (input_channels.max(2) / 2);
    let (mut input_producer, mut input_consumer) = HeapRb::<f32>::new(input_buffer_size).split();

    engine.input_channels = input_channels;

    let flag = Arc::clone(device_lost);
    let input_stream = input_device.and_then(|input_dev| {
        let input_config = input_dev.default_input_config().ok()?;
        let flag = Arc::clone(&flag);
        let stream = input_dev
            .build_input_stream(
                &input_config.into(),
                move |data: &[f32], _| {
                    input_producer.push_slice(data);
                },
                move |err| {
                    eprintln!("input stream error: {err}");
                    flag.store(true, Ordering::Release);
                },
                None,
            )
            .ok()?;
        stream.play().ok()?;
        Some(stream)
    });

    let device = match &args.output {
        Some(spec) => host
            .output_devices()
            .ok()
            .and_then(|d| find_device(d, spec))
            .unwrap_or_else(|| panic!("output device '{spec}' not found")),
        None => host.default_output_device().expect("no output device"),
    };

    let flag = Arc::clone(device_lost);
    let mut live_scratch = vec![0.0f32; 1024];
    let nch_in = input_channels.max(1);

    let output_stream = device
        .build_output_stream(
            config,
            move |data: &mut [f32], _| {
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        AudioCmd::Evaluate(s) => {
                            engine.evaluate(&s);
                        }
                        AudioCmd::Hush => engine.hush(),
                        AudioCmd::Panic => engine.panic(),
                    }
                }

                let buffer_samples = data.len() / output_channels;
                let raw_len = buffer_samples * nch_in;
                if live_scratch.len() < raw_len {
                    live_scratch.resize(raw_len, 0.0);
                }
                live_scratch[..raw_len].fill(0.0);
                input_consumer.pop_slice(&mut live_scratch[..raw_len]);

                engine.process_block(data, &[], &live_scratch[..raw_len]);
            },
            move |err| {
                eprintln!("stream error: {err}");
                flag.store(true, Ordering::Release);
            },
            None,
        )
        .unwrap();

    output_stream.play().unwrap();

    println!(
        "Output: {} @ {}Hz, {}ch",
        device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_default(),
        sample_rate as u32,
        output_channels
    );

    Streams {
        _output: output_stream,
        _input: input_stream,
    }
}
