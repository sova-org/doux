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
use doux::audio::{get_host, print_diagnostics, HostSelection};
use doux::cli_common::{
    build_audio_streams, print_devices, print_hosts, recreate_engine, resolve_output_config,
    StreamParams,
};
use doux::AudioCmd;
use doux::Engine;
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
        if let Some(idx) = line.find("//") {
            let before = &line[..idx];
            let comment = &line[idx..];
            let highlighted_before = highlight_pattern(before);
            return Cow::Owned(format!("{highlighted_before}{DIM_GRAY}{comment}{RESET}"));
        }

        if line.trim_start().starts_with('.') {
            return Cow::Owned(format!("{CYAN}{line}{RESET}"));
        }

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
            let mut token = String::new();
            token.push(c);
            while let Some(&next) = chars.peek() {
                if next == '/' {
                    break;
                }
                token.push(chars.next().unwrap());
            }
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

    let host_selection: HostSelection = args.host.parse().unwrap_or_else(|e| panic!("{e}"));

    if args.diagnose {
        print_hosts();
        println!();
        print_diagnostics();
        return Ok(());
    }

    let host = get_host(host_selection).unwrap_or_else(|e| panic!("{e}"));

    if args.list_devices {
        print_devices(&host);
        return Ok(());
    }

    let oc = resolve_output_config(
        &host,
        args.output.as_deref(),
        args.channels,
        args.buffer_size,
    );

    println!("doux-repl ({})", host.id().name());
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

    let stream_params = StreamParams {
        host: &host,
        input_spec: args.input.as_deref(),
        output_spec: args.output.as_deref(),
        config: &oc,
        device_lost: &device_lost,
    };

    let mut streams = build_audio_streams(&stream_params, engine, cmd_rx);

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
            drop(streams);
            std::thread::sleep(std::time::Duration::from_secs(1));

            let engine = recreate_engine(
                oc.sample_rate,
                oc.output_channels,
                max_voices,
                block_size,
                &sample_index,
                &sample_registry,
                #[cfg(feature = "soundfont")]
                &gm_bank,
            );
            metrics = Arc::clone(&engine.metrics);
            let (new_tx, new_rx) = crossbeam_channel::unbounded::<AudioCmd>();
            cmd_tx = new_tx;

            streams = build_audio_streams(&stream_params, engine, new_rx);
            eprintln!("Audio device reconnected");
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
