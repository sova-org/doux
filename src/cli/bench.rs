//! Native offline benchmarking for doux.

use clap::{Parser, Subcommand};
use doux::benchmark::{find_case, run_case, run_suite, BenchmarkOverrides, BenchmarkResult};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "doux-bench")]
#[command(about = "Benchmark native audio engine workloads", long_about = None)]
struct Args {
    /// Emit JSON instead of human-readable output.
    #[arg(long, global = true)]
    json: bool,

    /// Print internal phase timing breakdowns.
    #[arg(long, global = true)]
    breakdown: bool,

    /// Directory containing audio samples for cases that require them.
    #[arg(short, long, global = true)]
    samples: Option<PathBuf>,

    /// Number of fresh-engine repeats to aggregate.
    #[arg(long, default_value = "1", global = true)]
    repeats: usize,

    /// Override warmup duration in seconds.
    #[arg(long, global = true)]
    warmup: Option<f32>,

    /// Override measured duration in seconds.
    #[arg(long, global = true)]
    duration: Option<f32>,

    /// Override sample rate.
    #[arg(long, global = true)]
    sample_rate: Option<u32>,

    /// Override engine block size.
    #[arg(long, global = true)]
    block_size: Option<usize>,

    /// Override engine max voices.
    #[arg(long, global = true)]
    max_voices: Option<usize>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the full checked-in benchmark corpus.
    Suite,
    /// Run a single benchmark case.
    Case {
        /// Benchmark case name.
        name: String,
    },
}

fn main() {
    let args = Args::parse();
    let overrides = BenchmarkOverrides {
        repeats: args.repeats,
        warmup_seconds: args.warmup,
        measure_seconds: args.duration,
        sample_rate: args.sample_rate.map(|rate| rate as f32),
        block_size: args.block_size,
        max_voices: args.max_voices,
    };

    let result = match args.command {
        Command::Suite => run_suite(&overrides, args.samples.as_deref())
            .map(Output::Suite),
        Command::Case { name } => {
            let case = find_case(&name).ok_or_else(|| format!("unknown benchmark case '{name}'"));
            case.and_then(|case| run_case(case, &overrides, args.samples.as_deref()))
                .map(Output::Case)
        }
    };

    let output = match result {
        Ok(output) => output,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).expect("failed to serialize benchmark output")
        );
        return;
    }

    match output {
        Output::Case(result) => print_result(&result, args.breakdown),
        Output::Suite(results) => {
            for result in &results {
                print_result(result, args.breakdown);
                println!();
            }
        }
    }

    if args.breakdown && !cfg!(feature = "profiling") {
        eprintln!("Breakdown requested, but this build does not have the 'profiling' feature enabled.");
    }
}

#[derive(serde::Serialize)]
#[serde(untagged)]
enum Output {
    Case(BenchmarkResult),
    Suite(Vec<BenchmarkResult>),
}

fn print_result(result: &BenchmarkResult, show_breakdown: bool) {
    println!("{}", result.name);
    println!("{}", result.intent);
    println!(
        "repeats={} audio={:.2}s wall={:.3}ms rt={:.2}x ns/sample={:.1} ns/block={:.1}",
        result.repeats,
        result.audio_seconds,
        result.elapsed_ns as f64 / 1_000_000.0,
        result.realtime_factor,
        result.ns_per_sample,
        result.ns_per_block,
    );

    if show_breakdown {
        for phase in &result.breakdown {
            println!(
                "  {:<18} {:>7.2}% total_ns={:<12} calls={:<8} ns/sample={:.2}",
                phase.label,
                phase.percent_total,
                phase.total_ns,
                phase.calls,
                phase.ns_per_sample,
            );
        }
    }
}
