//! Native benchmark case corpus and offline runner.

use crate::offline::{
    apply_setup_commands, create_engine, run_without_capture, OfflineEngineConfig,
};
use crate::telemetry::{PhaseSummary, ProfilingSnapshot};
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct BenchmarkCase {
    pub name: &'static str,
    pub intent: &'static str,
    pub config: OfflineEngineConfig,
    pub warmup_seconds: f32,
    pub measure_seconds: f32,
    pub requires_samples: bool,
    pub setup_commands: fn() -> Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct BenchmarkOverrides {
    pub repeats: usize,
    pub warmup_seconds: Option<f32>,
    pub measure_seconds: Option<f32>,
    pub sample_rate: Option<f32>,
    pub block_size: Option<usize>,
    pub max_voices: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub intent: String,
    pub repeats: usize,
    pub audio_seconds: f64,
    pub elapsed_ns: u64,
    pub samples: u64,
    pub blocks: u64,
    pub ns_per_sample: f64,
    pub ns_per_block: f64,
    pub realtime_factor: f64,
    pub profiling_enabled: bool,
    pub breakdown: Vec<PhaseSummary>,
}

impl BenchmarkOverrides {
    pub fn repeats(&self) -> usize {
        self.repeats.max(1)
    }
}

pub fn benchmark_cases() -> &'static [BenchmarkCase] {
    &BENCHMARK_CASES
}

pub fn find_case(name: &str) -> Option<&'static BenchmarkCase> {
    benchmark_cases().iter().find(|case| case.name == name)
}

pub fn run_case(
    case: &BenchmarkCase,
    overrides: &BenchmarkOverrides,
    samples_dir: Option<&Path>,
) -> Result<BenchmarkResult, String> {
    if case.requires_samples && samples_dir.is_none() {
        return Err(format!("benchmark case '{}' requires --samples", case.name));
    }

    let repeats = overrides.repeats();
    let config = resolved_config(case, overrides);
    let warmup_seconds = overrides.warmup_seconds.unwrap_or(case.warmup_seconds);
    let measure_seconds = overrides.measure_seconds.unwrap_or(case.measure_seconds);

    let mut total_elapsed_ns = 0u64;
    let mut total_samples = 0u64;
    let mut total_blocks = 0u64;
    let mut breakdown = ProfilingSnapshot::default();

    for _ in 0..repeats {
        let mut engine = create_engine(config, samples_dir)?;
        apply_setup_commands(&mut engine, (case.setup_commands)());

        if warmup_seconds > 0.0 {
            let _ = run_without_capture(&mut engine, warmup_seconds);
        }

        engine.metrics.reset_profiling();
        let pass = run_without_capture(&mut engine, measure_seconds);
        total_elapsed_ns += pass.elapsed_ns;
        total_samples += pass.samples as u64;
        total_blocks += pass.blocks as u64;
        breakdown.merge_assign(&engine.metrics.profiling_snapshot());
    }

    let elapsed_secs = total_elapsed_ns as f64 / 1_000_000_000.0;
    let audio_seconds = total_samples as f64 / config.sample_rate as f64;

    Ok(BenchmarkResult {
        name: case.name.to_string(),
        intent: case.intent.to_string(),
        repeats,
        audio_seconds,
        elapsed_ns: total_elapsed_ns,
        samples: total_samples,
        blocks: total_blocks,
        ns_per_sample: if total_samples == 0 {
            0.0
        } else {
            total_elapsed_ns as f64 / total_samples as f64
        },
        ns_per_block: if total_blocks == 0 {
            0.0
        } else {
            total_elapsed_ns as f64 / total_blocks as f64
        },
        realtime_factor: if elapsed_secs == 0.0 {
            0.0
        } else {
            audio_seconds / elapsed_secs
        },
        profiling_enabled: cfg!(feature = "profiling"),
        breakdown: breakdown.sorted_summaries(),
    })
}

pub fn run_suite(
    overrides: &BenchmarkOverrides,
    samples_dir: Option<&Path>,
) -> Result<Vec<BenchmarkResult>, String> {
    benchmark_cases()
        .iter()
        .map(|case| run_case(case, overrides, samples_dir))
        .collect()
}

fn resolved_config(case: &BenchmarkCase, overrides: &BenchmarkOverrides) -> OfflineEngineConfig {
    let mut config = case.config;
    if let Some(sample_rate) = overrides.sample_rate {
        config.sample_rate = sample_rate;
    }
    if let Some(block_size) = overrides.block_size {
        config.block_size = block_size;
    }
    if let Some(max_voices) = overrides.max_voices {
        config.max_voices = max_voices;
    }
    config
}

const DEFAULT_CONFIG: OfflineEngineConfig = OfflineEngineConfig {
    sample_rate: 48_000.0,
    channels: 2,
    max_voices: 64,
    block_size: 512,
};

const STRESS_CONFIG: OfflineEngineConfig = OfflineEngineConfig {
    sample_rate: 48_000.0,
    channels: 2,
    max_voices: 128,
    block_size: 512,
};

const BENCHMARK_CASES: [BenchmarkCase; 10] = [
    BenchmarkCase {
        name: "baseline_sine",
        intent: "Simple baseline: one sustained sine voice with no filters or effects.",
        config: DEFAULT_CONFIG,
        warmup_seconds: 0.2,
        measure_seconds: 1.0,
        requires_samples: false,
        setup_commands: baseline_sine_commands,
    },
    BenchmarkCase {
        name: "oscillator_stack",
        intent: "Simple-to-mid complexity oscillator mix covering tri, saw, pulse, and morphing osc.",
        config: DEFAULT_CONFIG,
        warmup_seconds: 0.3,
        measure_seconds: 1.5,
        requires_samples: false,
        setup_commands: oscillator_stack_commands,
    },
    BenchmarkCase {
        name: "subtractive_pad",
        intent: "Representative musical texture with chord scheduling, filters, chorus, and reverb.",
        config: DEFAULT_CONFIG,
        warmup_seconds: 0.75,
        measure_seconds: 4.0,
        requires_samples: false,
        setup_commands: subtractive_pad_commands,
    },
    BenchmarkCase {
        name: "drum_bus",
        intent: "Representative rhythmic bus with dense transient scheduling and orbit effects.",
        config: DEFAULT_CONFIG,
        warmup_seconds: 0.5,
        measure_seconds: 4.0,
        requires_samples: false,
        setup_commands: drum_bus_commands,
    },
    BenchmarkCase {
        name: "modulation_showcase",
        intent: "Representative melodic case covering FM, vibrato, AM, RM, and phaser/flanger movement.",
        config: DEFAULT_CONFIG,
        warmup_seconds: 0.4,
        measure_seconds: 2.0,
        requires_samples: false,
        setup_commands: modulation_showcase_commands,
    },
    BenchmarkCase {
        name: "filter_matrix",
        intent: "Tone-shaping case covering SVF filters, ladder filters, EQ, and tilt.",
        config: DEFAULT_CONFIG,
        warmup_seconds: 0.4,
        measure_seconds: 1.75,
        requires_samples: false,
        setup_commands: filter_matrix_commands,
    },
    BenchmarkCase {
        name: "noise_texture",
        intent: "Noise-source case covering white, pink, and brown sources with smear and space effects.",
        config: DEFAULT_CONFIG,
        warmup_seconds: 0.5,
        measure_seconds: 1.75,
        requires_samples: false,
        setup_commands: noise_texture_commands,
    },
    BenchmarkCase {
        name: "sidechain_bus",
        intent: "Bus-mixing case focused on orbit routing, compressor sidechain, width, and spatial FX.",
        config: DEFAULT_CONFIG,
        warmup_seconds: 0.4,
        measure_seconds: 2.0,
        requires_samples: false,
        setup_commands: sidechain_bus_commands,
    },
    BenchmarkCase {
        name: "voice_stress",
        intent: "Synthetic polyphony stress focused on source generation and per-voice processing.",
        config: STRESS_CONFIG,
        warmup_seconds: 0.5,
        measure_seconds: 2.5,
        requires_samples: false,
        setup_commands: voice_stress_commands,
    },
    BenchmarkCase {
        name: "fx_stress",
        intent: "Synthetic sustained-orbit stress focused on delay, reverb, comb, feedback, and mix.",
        config: STRESS_CONFIG,
        warmup_seconds: 0.75,
        measure_seconds: 3.0,
        requires_samples: false,
        setup_commands: fx_stress_commands,
    },
];

fn baseline_sine_commands() -> Vec<String> {
    vec!["/time/0.000/sound/sine/note/60/gain/0.25/gate/1.4/release/0.1".to_string()]
}

fn oscillator_stack_commands() -> Vec<String> {
    let mut commands = Vec::new();
    let pattern = [
        (0.00, "tri", 48, "/gain/0.18/spread/8/gate/0.9/release/0.1"),
        (0.18, "saw", 55, "/gain/0.16/sub/0.35/gate/0.8/release/0.1"),
        (
            0.36,
            "pulse",
            60,
            "/pw/0.22/gain/0.14/width/1.2/gate/0.7/release/0.1",
        ),
        (
            0.54,
            "osc",
            67,
            "/wave/0.72/gain/0.15/spread/5/gate/0.8/release/0.1",
        ),
        (
            0.72,
            "pulze",
            72,
            "/pw/0.35/gain/0.10/haas/7/gate/0.6/release/0.1",
        ),
        (0.90, "sine", 79, "/gain/0.12/gate/0.55/release/0.08"),
    ];

    for (time, sound, note, suffix) in pattern {
        commands.push(format!("/time/{time:.3}/sound/{sound}/note/{note}{suffix}"));
    }

    commands
}

fn subtractive_pad_commands() -> Vec<String> {
    let mut commands = Vec::new();
    let chords = [
        [48, 55, 60],
        [50, 57, 62],
        [52, 59, 64],
        [45, 52, 57],
        [47, 54, 59],
        [43, 50, 55],
    ];

    for (step, chord) in chords.iter().enumerate() {
        let time = step as f32 * 0.75;
        for (voice, note) in chord.iter().enumerate() {
            commands.push(format!(
                "/time/{time:.3}/sound/saw/note/{note}/orbit/{}/spread/18/lpf/2800/lpq/0.4/chorus/0.25/chorusdepth/0.7/verb/0.6/verbdecay/0.8/verbdamp/0.35/width/1.4/gain/0.18/gate/1.6/release/1.1",
                voice % 2
            ));
        }
    }

    commands
}

fn drum_bus_commands() -> Vec<String> {
    let mut commands = Vec::new();
    for step in 0..24 {
        let time = step as f32 * 0.125;
        commands.push(format!(
            "/time/{time:.3}/sound/hat/orbit/1/gain/0.11/verb/0.08/decay/0.06/release/0.01"
        ));
        if step % 2 == 1 {
            commands.push(format!(
                "/time/{time:.3}/sound/rim/orbit/1/gain/0.09/decay/0.05/release/0.01"
            ));
        }
        if step % 4 == 0 {
            commands.push(format!(
                "/time/{time:.3}/sound/kick/orbit/0/freq/48/morph/0.55/harmonics/0.45/gain/0.5/comp/0.0/decay/0.36/release/0.01"
            ));
        }
        if step % 8 == 4 {
            commands.push(format!(
                "/time/{time:.3}/sound/snare/orbit/1/freq/180/timbre/0.7/harmonics/0.55/delay/0.2/delaytime/0.11/delayfeedback/0.3/gain/0.22/decay/0.18/release/0.02"
            ));
        }
        if step % 12 == 6 {
            commands.push(format!(
                "/time/{time:.3}/sound/tom/orbit/2/freq/105/morph/0.4/timbre/0.35/gain/0.18/decay/0.24/release/0.02"
            ));
        }
        if step % 16 == 8 {
            commands.push(format!(
                "/time/{time:.3}/sound/cowbell/orbit/2/gain/0.1/decay/0.11/release/0.01"
            ));
        }
        if step % 24 == 18 {
            commands.push(format!(
                "/time/{time:.3}/sound/cymbal/orbit/3/gain/0.08/verb/0.28/decay/0.55/release/0.03"
            ));
        }
    }
    commands.push(
        "/time/0.000/sound/saw/note/36/orbit/2/gate/4.0/release/0.4/gain/0.0/comp/0.65/comporbit/0"
            .to_string(),
    );
    commands
}

fn modulation_showcase_commands() -> Vec<String> {
    let mut commands = Vec::new();
    let notes = [60, 64, 67, 71, 74, 79];

    for (idx, note) in notes.iter().enumerate() {
        let time = idx as f32 * 0.18;
        commands.push(format!(
            "/time/{time:.3}/sound/osc/note/{note}/wave/0.15~0.92:0.7/vib/5.2/vibmod/0.15/am/3.5/amdepth/0.4/rm/11/rmdepth/0.22/phaser/0.7/phaserdepth/0.6/phasersweep/1800/phasercenter/900/gain/0.12/gate/1.2/release/0.25"
        ));
        commands.push(format!(
            "/time/{:.3}/sound/add/note/{}/partials/12/timbre/0.6/morph/0.4/harmonics/0.35/fm/1.3/fmh/2/fm2/0.4/fm2h/3/fmfb/0.08/flanger/0.22/flangerdepth/0.5/flangerfeedback/0.45/gain/0.08/gate/0.8/release/0.2",
            time + 0.06,
            note + 12
        ));
    }

    commands
}

fn filter_matrix_commands() -> Vec<String> {
    vec![
        "/time/0.000/sound/saw/note/48/lpf/1800/lpq/0.45/gain/0.10/gate/2.4/release/0.2".to_string(),
        "/time/0.020/sound/pulse/note/55/pw/0.18/hpf/420/hpq/0.35/gain/0.08/gate/2.2/release/0.2".to_string(),
        "/time/0.040/sound/tri/note/62/bpf/1200/bpq/0.55/gain/0.10/gate/2.0/release/0.2".to_string(),
        "/time/0.060/sound/saw/note/67/llpf/1200/llpq/0.72/gain/0.08/gate/2.3/release/0.2".to_string(),
        "/time/0.080/sound/saw/note/72/lhpf/350/lhpq/0.55/gain/0.08/gate/2.1/release/0.2".to_string(),
        "/time/0.100/sound/osc/note/79/wave/0.7/lbpf/900/lbpq/0.7/eqlo/4/eqmid/-3/eqhi/5/tilt/0.35/gain/0.07/gate/2.0/release/0.2".to_string(),
    ]
}

fn noise_texture_commands() -> Vec<String> {
    vec![
        "/time/0.000/sound/white/orbit/0/lpf/6200/smear/0.55/smearfreq/1800/smearfb/0.4/verb/0.45/gain/0.06/gate/3.0/release/0.3".to_string(),
        "/time/0.020/sound/pink/orbit/1/bpf/1400/bpq/0.35/eqlo/-3/eqmid/2/eqhi/4/chorus/0.18/chorusdepth/0.55/gain/0.05/gate/3.0/release/0.3".to_string(),
        "/time/0.040/sound/brown/orbit/2/hpf/120/tilt/-0.75/delay/0.25/delaytime/0.18/delayfeedback/0.35/gain/0.08/gate/3.0/release/0.3".to_string(),
    ]
}

fn sidechain_bus_commands() -> Vec<String> {
    let mut commands = Vec::new();
    for step in 0..16 {
        let time = step as f32 * 0.25;
        if step % 4 == 0 {
            commands.push(format!(
                "/time/{time:.3}/sound/kick/orbit/0/freq/47/gain/0.52/decay/0.34/release/0.02"
            ));
        }
        if step % 8 == 4 {
            commands.push(format!(
                "/time/{time:.3}/sound/snare/orbit/0/gain/0.18/decay/0.12/release/0.02"
            ));
        }
    }
    commands.push("/time/0.000/sound/saw/note/36/orbit/1/spread/16/chorus/0.24/verb/0.32/comp/0.72/comporbit/0/width/1.55/gain/0.10/gate/4.2/release/0.45".to_string());
    commands.push("/time/0.030/sound/pulse/note/43/orbit/1/pw/0.28/delay/0.32/delaytime/0.16/delayfeedback/0.42/comp/0.68/comporbit/0/haas/8/gain/0.08/gate/4.0/release/0.45".to_string());
    commands
}

fn voice_stress_commands() -> Vec<String> {
    let mut commands = Vec::new();
    let notes = [36, 43, 48, 55, 60, 67, 72, 79];

    for burst in 0..12 {
        let burst_time = burst as f32 * 0.12;
        for (idx, note) in notes.iter().enumerate() {
            let time = burst_time + idx as f32 * 0.004;
            commands.push(format!(
                "/time/{time:.3}/sound/add/note/{note}/partials/24/timbre/0.72/morph/0.38/harmonics/0.84/fm/1.7/fmh/2/fm2/1.15/fm2h/3/fmfb/0.16/spread/10/gain/0.11/gate/1.2/release/0.45"
            ));
        }
    }

    commands
}

fn fx_stress_commands() -> Vec<String> {
    let mut commands = Vec::new();
    let voices = [
        "/time/0.000/sound/white/orbit/0/lpf/5000/smear/0.7/smearfreq/2200/smearfb/0.7/verb/0.7/verbdecay/0.9/verbdamp/0.25/gain/0.05/gate/6.0/release/0.4",
        "/time/0.000/sound/saw/orbit/1/note/36/flanger/0.35/flangerdepth/0.8/flangerfeedback/0.75/delay/0.65/delaytime/0.22/delayfeedback/0.72/gain/0.14/gate/6.0/release/0.4",
        "/time/0.020/sound/pulse/orbit/2/note/48/pw/0.22/feedback/0.7/fbtime/180/fbdamp/0.5/comb/0.55/combfreq/330/combfeedback/0.92/combdamp/0.25/gain/0.12/gate/6.0/release/0.4",
        "/time/0.040/sound/tri/orbit/3/note/55/chorus/0.3/chorusdepth/0.75/chorusdelay/24/verb/0.55/verbtype/plate/verbdecay/0.8/verbdamp/0.4/gain/0.12/gate/6.0/release/0.4",
        "/time/0.060/sound/saw/orbit/1/note/67/comp/0.75/comporbit/0/llpf/1800/llpq/0.7/haas/10/width/1.6/gain/0.08/gate/6.0/release/0.4",
    ];

    commands.extend(voices.into_iter().map(str::to_string));
    commands
}
