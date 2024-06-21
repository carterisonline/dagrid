use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use dagrid_core::control::ControlGraph;
use dagrid_core::presets::{self, preset};

fn construct(c: &mut Criterion) {
    let mut g = c.benchmark_group("construct");
    g.warm_up_time(Duration::from_millis(500));
    g.measurement_time(Duration::from_millis(500));

    g.bench_function("subsynth_plain", |b| {
        b.iter(|| preset(48000, presets::subsynth_plain))
    });

    g.bench_function("subsynth_with_containers", |b| {
        b.iter(|| preset(48000, presets::subsynth_with_containers))
    });

    g.finish();
}

fn synth(c: &mut Criterion) {
    let mut g = c.benchmark_group("synth");
    g.warm_up_time(Duration::from_secs(2));
    g.measurement_time(Duration::from_secs(3));
    g.sample_size(300);

    g.bench_function("subsynth_plain", |b| {
        fn subsynth_plain_x(cg: &mut ControlGraph) {
            for _ in 0..48000 {
                black_box(cg.next_sample());
            }
        }

        b.iter_batched(
            || preset(48000, presets::subsynth_plain),
            |mut cg| subsynth_plain_x(&mut cg),
            BatchSize::SmallInput,
        )
    });

    g.bench_function("subsynth_with_containers", |b| {
        fn subsynth_with_containers_x(cg: &mut ControlGraph) {
            for _ in 0..48000 {
                black_box(cg.next_sample());
            }
        }

        b.iter_batched(
            || preset(48000, presets::subsynth_with_containers),
            |mut cg| subsynth_with_containers_x(&mut cg),
            BatchSize::SmallInput,
        )
    });

    g.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = synth, construct
}

criterion_main!(benches);
