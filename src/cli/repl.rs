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
//!       --host <HOST>       Audio host: jack, alsa, asio, or auto (default: auto)
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
//! | `.patch`  |       | Install an arf patch (`.patch <name> <source>`) |
//! | `.help`   | `.h`  | Show available commands              |
//!
//! Any other input is evaluated as a doux pattern.

use clap::Parser;
use crossbeam_channel::TrySendError;
use doux::cli_common::{
    build_audio_streams, init_audio_host, setup_engine_samples, CommonAudioArgs, HostInit,
    StreamParams,
};
use doux::event::Event;
use doux::types::AUDIO_CMD_QUEUE_DEPTH;
use doux::{AudioCmd, Engine, EngineConfig, EngineMetrics};
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
    #[command(flatten)]
    common: CommonAudioArgs,
}

fn print_help() {
    println!("Commands:");
    println!("  .quit, .q    Exit the REPL");
    println!("  .reset, .r   Reset engine state");
    println!("  .hush        Fade out all voices");
    println!("  .panic       Immediately silence all voices");
    println!("  .voices      Show active voice count");
    println!("  .time        Show engine time");
    println!("  .stats, .s   Show engine telemetry (load, voices, memory)");
    println!("  .patch <name> <arf source>");
    println!("               Compile + install an arf patch (play it as s/<name>)");
    println!("  .help, .h    Show this help");
    println!();
    println!("Any other input is evaluated as a doux pattern.");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let (host, oc, buffer_size) = match init_audio_host(&args.common)? {
        HostInit::Ready {
            host,
            output_config,
            buffer_size,
        } => (host, output_config, buffer_size),
        HostInit::EarlyExit => return Ok(()),
    };

    println!("doux-repl ({})", host.id().name());
    if let Some(buf) = args.common.buffer_size {
        let latency_ms = buf as f32 / oc.sample_rate * 1000.0;
        println!("Buffer: {buf} samples ({latency_ms:.1} ms)");
    }

    let mut engine = Engine::new(EngineConfig {
        sample_rate: oc.sample_rate,
        output_channels: oc.output_channels,
        max_voices: args.common.max_voices,
        host_buffer_size: buffer_size,
        inner_block_size: args.common.dsp_block_size,
        metrics: Arc::new(EngineMetrics::default()),
        sample_registry: None,
        patch_registry: None,
    });

    if let Some(ref dir) = args.common.samples {
        setup_engine_samples(&mut engine, dir, false, false);
        println!(
            "Samples: {} from {}",
            engine.sample_index().len(),
            dir.display()
        );
    }

    let sample_index = engine.sample_index().to_vec();
    let sample_registry = Arc::clone(engine.sample_registry());
    let patch_registry = Arc::clone(engine.patch_registry());
    #[cfg(feature = "soundfont")]
    let gm_bank = engine.gm_bank();
    let max_voices = args.common.max_voices;
    let dsp_block_size = args.common.dsp_block_size;
    let mut metrics = Arc::clone(engine.metrics());

    let device_lost = Arc::new(AtomicBool::new(false));

    let (mut cmd_tx, cmd_rx) = crossbeam_channel::bounded::<AudioCmd>(AUDIO_CMD_QUEUE_DEPTH);

    let stream_params = StreamParams {
        host: &host,
        input_spec: args.common.input.as_deref(),
        output_spec: args.common.output.as_deref(),
        config: &oc,
        device_lost: &device_lost,
    };

    let mut streams = build_audio_streams(&stream_params, engine, cmd_rx)?;

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

            let engine = Engine::new(EngineConfig {
                sample_rate: oc.sample_rate,
                output_channels: oc.output_channels,
                max_voices,
                host_buffer_size: buffer_size,
                inner_block_size: dsp_block_size,
                metrics: Arc::new(EngineMetrics::default()),
                sample_registry: Some(Arc::clone(&sample_registry)),
                patch_registry: Some(Arc::clone(&patch_registry)),
            });
            engine.set_sample_index(sample_index.clone());
            #[cfg(feature = "soundfont")]
            if let Some(bank) = gm_bank.clone() {
                engine.set_gm_bank(bank);
            }
            metrics = Arc::clone(engine.metrics());
            let (new_tx, new_rx) = crossbeam_channel::bounded::<AudioCmd>(AUDIO_CMD_QUEUE_DEPTH);
            cmd_tx = new_tx;

            match build_audio_streams(&stream_params, engine, new_rx) {
                Ok(s) => {
                    streams = s;
                    eprintln!("Audio device reconnected");
                }
                Err(e) => {
                    eprintln!("{RED}[error]{RESET} Failed to reconnect: {e}");
                    return Err(e.into());
                }
            }
        }
        match rl.readline("doux> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                let trimmed = line.trim();

                let send_cmd = |cmd: AudioCmd| match cmd_tx.try_send(cmd) {
                    Ok(()) => {}
                    Err(TrySendError::Full(_)) => {
                        metrics.dropped_cmds.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(TrySendError::Disconnected(_)) => {}
                };
                match trimmed {
                    ".quit" | ".q" => break,
                    ".reset" | ".r" => {
                        let event = Event::parse("/doux/reset", oc.sample_rate);
                        send_cmd(AudioCmd::DispatchEvent(event));
                    }
                    ".voices" | ".v" => {
                        println!("{}", metrics.active_voices.load(Ordering::Relaxed));
                    }
                    ".time" | ".t" => {
                        let t = f64::from_bits(metrics.time_bits.load(Ordering::Relaxed));
                        println!("{t:.3}s");
                    }
                    ".stats" | ".s" => {
                        let load_pct = metrics.load.get_load() * 100.0;
                        let voices = metrics.active_voices.load(Ordering::Relaxed);
                        let peak = metrics.peak_voices.load(Ordering::Relaxed);
                        let sched = metrics.schedule_depth.load(Ordering::Relaxed);
                        let mem = metrics.sample_pool_mb();
                        println!("Load:     {load_pct:5.1}%");
                        println!("Voices:   {voices:3}/{max_voices}");
                        println!("Peak:     {peak:3}");
                        println!("Schedule: {sched:3}");
                        println!("Samples:  {mem:.1} MB");
                    }
                    ".hush" => {
                        send_cmd(AudioCmd::Hush);
                    }
                    ".panic" => {
                        send_cmd(AudioCmd::Panic);
                    }
                    ".help" | ".h" => {
                        print_help();
                    }
                    // A serialized patch graph (JSON) carries `/`, `:` and `{}`, so it can't
                    // ride the slash protocol — the dot-command takes the payload verbatim.
                    s if s == ".patch" || s.starts_with(".patch ") => {
                        let rest = s.strip_prefix(".patch").unwrap_or_default().trim_start();
                        match rest.split_once(char::is_whitespace) {
                            Some((name, graph_json)) if !graph_json.trim().is_empty() => {
                                match patch_registry.install_graph(name, graph_json, oc.sample_rate) {
                                    Ok(()) => println!("installed {name}"),
                                    Err(e) => eprintln!("{RED}[patch]{RESET} {e}"),
                                }
                            }
                            _ => eprintln!("usage: .patch <name> <graph json>"),
                        }
                    }
                    s if !s.is_empty() => {
                        let event = Event::parse(s, oc.sample_rate);
                        send_cmd(AudioCmd::DispatchEvent(event));
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
