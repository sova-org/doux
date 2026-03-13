//! Interactive REPL for the doux audio engine.
//!
//! Provides a command-line interface for live-coding audio patterns with
//! readline-style editing and persistent history.
//!
//! # Usage
//!
//! ```text
//! doux-repl [OPTIONS]
//!
//! Options:
//!   -s, --samples <PATH>    Directory containing audio samples
//!   -i, --input <DEVICE>    Input device (name or index)
//!   -o, --output <DEVICE>   Output device (name or index)
//!       --channels <N>      Number of output channels (default: 2)
//!       --list-devices      List available audio devices and exit
//!       --host <HOST>       Audio host: jack, alsa, or auto (default: auto)
//!       --diagnose          Run audio diagnostics and exit
//! ```
//!
//! # REPL Commands
//!
//! | Command   | Alias | Description                          |
//! |-----------|-------|--------------------------------------|
//! | `.quit`   | `.q`  | Exit the REPL                        |
//! | `.reset`  | `.r`  | Reset engine state                   |
//! | `.hush`   |       | Fade out all voices                  |
//! | `.panic`  |       | Immediately silence all voices       |
//! | `.voices` |       | Show active voice count              |
//! | `.time`   |       | Show engine time in seconds          |
//! | `.help`   | `.h`  | Show available commands              |
//!
//! Any other input is evaluated as a doux pattern.

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host};
use doux::audio::{get_host, list_hosts, print_diagnostics, HostSelection};
use doux::osc::AudioCmd;
use doux::Engine;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ANSI color codes
const RESET: &str = "\x1b[0m";
const GRAY: &str = "\x1b[90m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const DIM_GRAY: &str = "\x1b[2;90m";
const CYAN: &str = "\x1b[36m";

struct DouxHighlighter;

impl Highlighter for DouxHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Comment: everything after //
        if let Some(idx) = line.find("//") {
            let before = &line[..idx];
            let comment = &line[idx..];
            let highlighted_before = highlight_pattern(before);
            return Cow::Owned(format!("{highlighted_before}{DIM_GRAY}{comment}{RESET}"));
        }

        // Dot command
        if line.trim_start().starts_with('.') {
            return Cow::Owned(format!("{CYAN}{line}{RESET}"));
        }

        // Pattern with /key/value
        Cow::Owned(highlight_pattern(line))
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }
}

fn highlight_pattern(line: &str) -> String {
    let mut result = String::new();
    let mut chars = line.chars().peekable();
    let mut after_slash = false;

    while let Some(c) = chars.next() {
        if c == '/' {
            result.push_str(GRAY);
            result.push(c);
            result.push_str(RESET);
            after_slash = true;
        } else if after_slash {
            // Collect the token until next /
            let mut token = String::new();
            token.push(c);
            while let Some(&next) = chars.peek() {
                if next == '/' {
                    break;
                }
                token.push(chars.next().unwrap());
            }
            // Is it a number?
            if is_number(&token) {
                result.push_str(RED);
                result.push_str(&token);
                result.push_str(RESET);
            } else {
                result.push_str(BOLD);
                result.push_str(&token);
                result.push_str(RESET);
            }
            after_slash = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn is_number(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let s = s.strip_prefix('-').unwrap_or(s);
    s.chars().all(|c| c.is_ascii_digit() || c == '.')
}

impl Completer for DouxHighlighter {
    type Candidate = String;
}

impl Hinter for DouxHighlighter {
    type Hint = String;
}

impl Validator for DouxHighlighter {}

impl Helper for DouxHighlighter {}

/// Maximum samples buffered from audio input.
const INPUT_BUFFER_SIZE: usize = 8192;

#[derive(Parser)]
#[command(name = "doux-repl")]
#[command(about = "Interactive REPL for doux audio engine")]
struct Args {
    /// Directory containing audio samples
    #[arg(short, long)]
    samples: Option<PathBuf>,

    /// List available audio devices and exit
    #[arg(long)]
    list_devices: bool,

    /// Input device (name or index)
    #[arg(short, long)]
    input: Option<String>,

    /// Output device (name or index)
    #[arg(short, long)]
    output: Option<String>,

    /// Number of output channels (default: 2, max depends on device)
    #[arg(long, default_value = "2")]
    channels: u16,

    /// Audio buffer size in samples (lower = less latency, higher = more stable).
    /// Common values: 64, 128, 256, 512, 1024. Default: system choice.
    #[arg(short, long)]
    buffer_size: Option<u32>,

    /// Maximum polyphony (number of simultaneous voices).
    #[arg(long, default_value = "32")]
    max_voices: usize,

    /// Audio host backend: jack, alsa, or auto (default: auto).
    /// On Linux with PipeWire, use 'jack' for best compatibility.
    #[arg(long, default_value = "auto")]
    host: String,

    /// Run audio diagnostics and exit (useful for troubleshooting on Linux).
    #[arg(long)]
    diagnose: bool,
}

/// Prints available audio input and output devices.
///
/// Default devices are marked with `*`.
fn list_devices(host: &Host) {
    let default_in = host
        .default_input_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));
    let default_out = host
        .default_output_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));

    println!("Input devices:");
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
    }
}

/// Finds a device by index or substring match on name.
fn find_device<I>(devices: I, spec: &str) -> Option<Device>
where
    I: Iterator<Item = Device>,
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

/// Prints available REPL commands.
fn print_help() {
    println!("Commands:");
    println!("  .quit, .q    Exit the REPL");
    println!("  .reset, .r   Reset engine state");
    println!("  .hush        Fade out all voices");
    println!("  .panic       Immediately silence all voices");
    println!("  .voices      Show active voice count");
    println!("  .time        Show engine time");
    println!("  .stats, .s   Show engine telemetry (CPU, voices, memory)");
    println!("  .help, .h    Show this help");
    println!();
    println!("Any other input is evaluated as a doux pattern.");
}

fn print_hosts() {
    println!("Available audio hosts:");
    for h in list_hosts() {
        let status = if h.available { "" } else { " (unavailable)" };
        println!("  {}{}", h.name, status);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Parse host selection
    let host_selection: HostSelection = args.host.parse().unwrap_or_else(|e| panic!("{e}"));

    // Handle diagnose flag first
    if args.diagnose {
        print_hosts();
        println!();
        print_diagnostics();
        return Ok(());
    }

    // Get the audio host
    let host = get_host(host_selection).unwrap_or_else(|e| panic!("{e}"));

    if args.list_devices {
        list_devices(&host);
        return Ok(());
    }

    let (output_channels, sample_rate, config) = {
        let device = match &args.output {
            Some(spec) => host
                .output_devices()
                .ok()
                .and_then(|d| find_device(d, spec))
                .unwrap_or_else(|| panic!("output device '{spec}' not found")),
            None => host.default_output_device().expect("no output device"),
        };

        let max_channels = device
            .supported_output_configs()
            .map(|configs| configs.map(|c| c.channels()).max().unwrap_or(2))
            .unwrap_or(2);

        let output_channels = (args.channels as usize).min(max_channels as usize);
        if args.channels as usize > output_channels {
            eprintln!(
                "Warning: device supports max {} channels, using that instead of {}",
                max_channels, args.channels
            );
        }

        let default_config = device.default_output_config()?;
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

    println!("doux-repl ({})", host.id().name());
    if let Some(buf) = args.buffer_size {
        let latency_ms = buf as f32 / sample_rate * 1000.0;
        println!("Buffer: {buf} samples ({latency_ms:.1} ms)");
    }

    let block_size = args.buffer_size.map(|b| b as usize).unwrap_or(doux::types::DEFAULT_NATIVE_BLOCK_SIZE);
    let mut engine = Engine::new_with_channels(sample_rate, output_channels, args.max_voices, block_size);

    if let Some(ref dir) = args.samples {
        let index = doux::sampling::scan_samples_dir(dir);
        println!("Samples: {} from {}", index.len(), dir.display());
        engine.sample_index = index;

        #[cfg(feature = "soundfont")]
        engine.load_soundfont_from_dir(dir);
    }

    let sample_index = engine.sample_index.clone();
    let sample_registry = Arc::clone(&engine.sample_registry);
    #[cfg(feature = "soundfont")]
    let gm_bank = engine.gm_bank.clone();
    let max_voices = args.max_voices;
    let mut metrics = Arc::clone(&engine.metrics);

    let device_lost = Arc::new(AtomicBool::new(false));

    let (mut cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<AudioCmd>();
    let (mut input_stream, mut output_stream) = build_repl_streams(
        &host,
        &args,
        &config,
        output_channels,
        sample_rate,
        engine,
        cmd_rx,
        &device_lost,
    )?;

    let mut rl = rustyline::Editor::new()?;
    rl.set_helper(Some(DouxHighlighter));
    let history_path = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".doux_history"))
        .unwrap_or_else(|_| PathBuf::from(".doux_history"));
    let _ = rl.load_history(&history_path);

    println!("Type .help for commands");

    loop {
        if device_lost.load(Ordering::Acquire) {
            eprintln!("{RED}[error]{RESET} Audio device lost, reconnecting...");
            device_lost.store(false, Ordering::Release);
            drop(output_stream.take());
            drop(input_stream.take());
            std::thread::sleep(std::time::Duration::from_secs(1));

            // Recreate engine and channel
            let mut engine =
                Engine::new_with_channels(sample_rate, output_channels, max_voices, block_size);
            engine.sample_index = sample_index.clone();
            engine.sample_registry = Arc::clone(&sample_registry);
            #[cfg(feature = "soundfont")]
            {
                engine.gm_bank = gm_bank.clone();
            }
            metrics = Arc::clone(&engine.metrics);
            let (new_tx, new_rx) = crossbeam_channel::unbounded::<AudioCmd>();
            cmd_tx = new_tx;

            match build_repl_streams(
                &host,
                &args,
                &config,
                output_channels,
                sample_rate,
                engine,
                new_rx,
                &device_lost,
            ) {
                Ok((inp, out)) => {
                    input_stream = inp;
                    output_stream = out;
                    eprintln!("Audio device reconnected");
                }
                Err(e) => {
                    eprintln!("{RED}[error]{RESET} Reconnection failed: {e}");
                }
            }
        }
        match rl.readline("doux> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                let trimmed = line.trim();

                match trimmed {
                    ".quit" | ".q" => break,
                    ".reset" | ".r" => {
                        let _ = cmd_tx.send(AudioCmd::Evaluate("/doux/reset".into()));
                    }
                    ".voices" | ".v" => {
                        println!(
                            "{}",
                            metrics.active_voices.load(Ordering::Relaxed)
                        );
                    }
                    ".time" | ".t" => {
                        let t = f64::from_bits(metrics.time_bits.load(Ordering::Relaxed));
                        println!("{t:.3}s");
                    }
                    ".stats" | ".s" => {
                        let cpu = metrics.load.get_load() * 100.0;
                        let voices = metrics.active_voices.load(Ordering::Relaxed);
                        let peak = metrics.peak_voices.load(Ordering::Relaxed);
                        let sched = metrics.schedule_depth.load(Ordering::Relaxed);
                        let mem = metrics.sample_pool_mb();
                        println!("CPU:      {cpu:5.1}%");
                        println!("Voices:   {voices:3}/{max_voices}");
                        println!("Peak:     {peak:3}");
                        println!("Schedule: {sched:3}");
                        println!("Samples:  {mem:.1} MB");
                    }
                    ".hush" => {
                        let _ = cmd_tx.send(AudioCmd::Hush);
                    }
                    ".panic" => {
                        let _ = cmd_tx.send(AudioCmd::Panic);
                    }
                    ".help" | ".h" => {
                        print_help();
                    }
                    s if !s.is_empty() => {
                        let _ = cmd_tx.send(AudioCmd::Evaluate(s.into()));
                    }
                    _ => {}
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("readline error: {e}");
                break;
            }
        }
    }

    let _ = rl.save_history(&history_path);
    Ok(())
}

fn build_repl_streams(
    host: &Host,
    args: &Args,
    config: &cpal::StreamConfig,
    output_channels: usize,
    sample_rate: f32,
    mut engine: Engine,
    cmd_rx: crossbeam_channel::Receiver<AudioCmd>,
    device_lost: &Arc<AtomicBool>,
) -> Result<(Option<cpal::Stream>, Option<cpal::Stream>), Box<dyn std::error::Error>> {
    let input_device = match &args.input {
        Some(spec) => host.input_devices().ok().and_then(|d| find_device(d, spec)),
        None => doux::audio::default_input_device(),
    };

    let input_channels: usize = input_device
        .as_ref()
        .and_then(|dev| dev.default_input_config().ok())
        .map_or(0, |cfg| cfg.channels() as usize);

    let input_buffer_size = INPUT_BUFFER_SIZE * (input_channels.max(2) / 2);
    let (mut input_producer, mut input_consumer) = HeapRb::<f32>::new(input_buffer_size).split();

    engine.input_channels = input_channels;

    let flag = Arc::clone(device_lost);
    let input_stream = input_device.and_then(|input_dev| {
        let input_config = input_dev.default_input_config().ok()?;
        println!(
            "Input: {}",
            input_dev
                .description()
                .map(|d| d.name().to_string())
                .unwrap_or_default()
        );
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

    let sr = sample_rate;
    let ch = output_channels;
    let nch_in = input_channels.max(1);
    let flag = Arc::clone(device_lost);
    let mut scratch = vec![0.0f32; 4096];

    let output_stream = device.build_output_stream(
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

            let buffer_samples = data.len() / ch;
            let raw_len = buffer_samples * nch_in;
            if scratch.len() < raw_len {
                scratch.resize(raw_len, 0.0);
            }
            scratch[..raw_len].fill(0.0);
            input_consumer.pop_slice(&mut scratch[..raw_len]);

            let buffer_time_ns = (buffer_samples as f64 / sr as f64 * 1e9) as u64;
            engine.metrics.load.set_buffer_time(buffer_time_ns);
            engine.process_block(data, &[], &scratch[..raw_len]);
        },
        move |err| {
            eprintln!("stream error: {err}");
            flag.store(true, Ordering::Release);
        },
        None,
    )?;
    output_stream.play()?;

    println!(
        "Output: {} @ {}Hz, {}ch",
        device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_default(),
        sr as u32,
        ch,
    );

    Ok((input_stream, Some(output_stream)))
}
