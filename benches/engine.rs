use criterion::{criterion_group, criterion_main, Criterion};
use doux::benchmark::{benchmark_cases, run_case, BenchmarkOverrides};

fn engine_benches(c: &mut Criterion) {
    let overrides = BenchmarkOverrides {
        repeats: 1,
        ..BenchmarkOverrides::default()
    };

    for case in benchmark_cases() {
        c.bench_function(case.name, |b| {
            b.iter(|| {
                run_case(case, &overrides, None).expect("benchmark case should run");
            });
        });
    }
}

criterion_group!(benches, engine_benches);
criterion_main!(benches);
