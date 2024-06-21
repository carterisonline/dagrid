#![feature(macro_metavar_expr)]

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use dagrid_core::control::ControlGraph;
use dagrid_core::presets::{self, preset};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("subsynth_plain", |b| {
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

    c.bench_function("subsynth_with_containers", |b| {
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
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
