use doux::benchmark::{benchmark_cases, find_case, run_case, run_suite, BenchmarkOverrides};
use doux::offline::{
    apply_setup_commands, create_engine, render_to_buffer, seconds_to_samples, OfflineEngineConfig,
};

fn short_overrides() -> BenchmarkOverrides {
    BenchmarkOverrides {
        repeats: 1,
        warmup_seconds: Some(0.0),
        measure_seconds: Some(0.05),
        ..BenchmarkOverrides::default()
    }
}

#[test]
fn shared_offline_runner_matches_manual_render_loop() {
    let config = OfflineEngineConfig {
        sample_rate: 48_000.0,
        channels: 2,
        max_voices: 32,
        block_size: 128,
    };
    let commands = [
        "/sound/sine/note/60/gain/0.3/gate/0.4/release/0.1",
        "/time/0.2/sound/saw/note/67/gain/0.15/chorus/0.2/gate/0.3/release/0.1",
    ];
    let duration = 0.5;

    let mut manual = create_engine(config, None).unwrap();
    apply_setup_commands(&mut manual, commands);
    let mut expected = vec![0.0f32; seconds_to_samples(config.sample_rate, duration) * config.channels];
    for chunk in expected.chunks_mut(config.block_size * config.channels) {
        manual.process_block(chunk, &[], &[]);
    }

    let mut shared = create_engine(config, None).unwrap();
    apply_setup_commands(&mut shared, commands);
    let actual = render_to_buffer(&mut shared, duration)
        .output
        .expect("captured output");

    assert_eq!(expected.len(), actual.len());
    for (lhs, rhs) in expected.iter().zip(actual.iter()) {
        assert!((lhs - rhs).abs() < 1e-8);
    }
}

#[test]
fn benchmark_case_runs_and_returns_metrics() {
    let result = run_case(find_case("subtractive_pad").unwrap(), &short_overrides(), None).unwrap();

    assert_eq!(result.name, "subtractive_pad");
    assert!(result.samples > 0);
    assert!(result.blocks > 0);
    assert!(result.elapsed_ns > 0);
}

#[test]
fn benchmark_suite_runs_all_cases() {
    let results = run_suite(&short_overrides(), None).unwrap();

    assert_eq!(results.len(), benchmark_cases().len());
    assert!(results.iter().all(|result| result.samples > 0));
}
