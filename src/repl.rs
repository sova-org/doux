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
use doux::Engine;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
}

/// Prints available audio input and output devices.
///
/// Default devices are marked with `*`.
fn list_devices(host: &Host) {
    let default_in = host.default_input_device().and_then(|d| d.name().ok());
    let default_out = host.default_output_device().and_then(|d| d.name().ok());

    println!("Input devices:");
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
        d.name()
            .map(|n| n.to_lowercase().contains(&spec_lower))
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let host = cpal::default_host();

    if args.list_devices {
        list_devices(&host);
        return Ok(());
    }

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
    let sample_rate = default_config.sample_rate().0 as f32;

    let config = cpal::StreamConfig {
        channels: output_channels as u16,
        sample_rate: default_config.sample_rate(),
        buffer_size: args
            .buffer_size
            .map(cpal::BufferSize::Fixed)
            .unwrap_or(cpal::BufferSize::Default),
    };

    println!("doux-repl");
    print!(
        "Output: {} @ {}Hz, {} channels",
        device.name().unwrap_or_default(),
        sample_rate,
        output_channels
    );
    if let Some(buf) = args.buffer_size {
        let latency_ms = buf as f32 / sample_rate * 1000.0;
        println!(", {buf} samples ({latency_ms:.1} ms)");
    } else {
        println!();
    }

    let mut engine = Engine::new_with_channels(sample_rate, output_channels, args.max_voices);

    if let Some(ref dir) = args.samples {
        let index = doux::loader::scan_samples_dir(dir);
        println!("Samples: {} from {}", index.len(), dir.display());
        engine.sample_index = index;
    }

    let engine = Arc::new(Mutex::new(engine));
    let input_buffer: Arc<Mutex<VecDeque<f32>>> =
        Arc::new(Mutex::new(VecDeque::with_capacity(INPUT_BUFFER_SIZE)));

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
                    b.extend(data.iter().copied());
                    let excess = b.len().saturating_sub(INPUT_BUFFER_SIZE);
                    if excess > 0 {
                        drop(b.drain(..excess));
                    }
                },
                |err| eprintln!("input error: {err}"),
                None,
            )
            .ok()?;
        stream.play().ok()?;
        Some(stream)
    });

    let engine_clone = Arc::clone(&engine);
    let input_buf_clone = Arc::clone(&input_buffer);
    let sr = sample_rate;
    let ch = output_channels;

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _| {
            let mut scratch = vec![0.0f32; data.len()];
            {
                let mut buf = input_buf_clone.lock().unwrap();
                let available = buf.len().min(data.len());
                for (i, sample) in buf.drain(..available).enumerate() {
                    scratch[i] = sample;
                }
            }
            let mut engine = engine_clone.lock().unwrap();
            let buffer_samples = data.len() / ch;
            let buffer_time_ns = (buffer_samples as f64 / sr as f64 * 1e9) as u64;
            engine.metrics.load.set_buffer_time(buffer_time_ns);
            engine.process_block(data, &[], &scratch);
        },
        |err| eprintln!("stream error: {err}"),
        None,
    )?;
    stream.play()?;

    let mut rl = rustyline::Editor::new()?;
    rl.set_helper(Some(DouxHighlighter));
    let history_path = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".doux_history"))
        .unwrap_or_else(|_| PathBuf::from(".doux_history"));
    let _ = rl.load_history(&history_path);

    println!("Type .help for commands");

    loop {
        match rl.readline("doux> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                let trimmed = line.trim();

                match trimmed {
                    ".quit" | ".q" => break,
                    ".reset" | ".r" => {
                        engine.lock().unwrap().evaluate("/doux/reset");
                    }
                    ".voices" | ".v" => {
                        println!("{}", engine.lock().unwrap().active_voices);
                    }
                    ".time" | ".t" => {
                        println!("{:.3}s", engine.lock().unwrap().time);
                    }
                    ".stats" | ".s" => {
                        use std::sync::atomic::Ordering;
                        let e = engine.lock().unwrap();
                        let cpu = e.metrics.load.get_load() * 100.0;
                        let voices = e.metrics.active_voices.load(Ordering::Relaxed);
                        let peak = e.metrics.peak_voices.load(Ordering::Relaxed);
                        let sched = e.metrics.schedule_depth.load(Ordering::Relaxed);
                        let mem = e.metrics.sample_pool_mb();
                        println!("CPU:      {cpu:5.1}%");
                        println!("Voices:   {voices:3}/{}", e.max_voices);
                        println!("Peak:     {peak:3}");
                        println!("Schedule: {sched:3}");
                        println!("Samples:  {mem:.1} MB");
                    }
                    ".hush" => {
                        engine.lock().unwrap().hush();
                    }
                    ".panic" => {
                        engine.lock().unwrap().panic();
                    }
                    ".help" | ".h" => {
                        print_help();
                    }
                    s if !s.is_empty() => {
                        engine.lock().unwrap().evaluate(s);
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
