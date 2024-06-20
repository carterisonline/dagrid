#![feature(macro_metavar_expr)]

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use dagrid_core::container::*;
use dagrid_core::control::ControlGraph;
use dagrid_core::node::*;

fn subsynth_plain(sample_rate: u32) -> ControlGraph {
    let mut cg = ControlGraph::new(sample_rate);
    let s1 = cg.connect_const_new(440.0, Sine);
    let s2 = cg.connect_const_new(220.0, Sine);
    let mulneg1 = cg.connect_const_new(-1.0, Mul);
    cg.connect(s2, mulneg1, 1);
    let add = cg.connect_many_new(&[mulneg1, s1], Add);
    let mulhalf = cg.connect_const_new(0.5, Mul);
    cg.connect(add, mulhalf, 1);
    cg.connect_existing_aout(mulhalf);

    cg
}

fn subsynth_with_containers(sample_rate: u32) -> ControlGraph {
    let mut cg = ControlGraph::new(sample_rate);

    let s1 = cg.connect_const_new(440.0, Sine);
    let s2 = cg.connect_const_new(220.0, Sine);

    let (sub_in, sub_out) = cg.insert_container(Sub);
    cg.connect_ex_ex(s1, sub_in[0]);
    cg.connect_ex_ex(s2, sub_in[1]);

    let (div_in, div_out) = cg.insert_container(Div);
    cg.connect_ex_ex(sub_out[0], div_in[0]);
    cg.connect_const_ex(2.0, div_in[1]);

    cg.connect_existing_aout(div_out[0]);

    cg
}

fn subsynth_plain_x(cg: &mut ControlGraph) {
    for _ in 0..48000 {
        black_box(cg.next_sample());
    }
}

fn subsynth_with_containers_x(cg: &mut ControlGraph) {
    for _ in 0..48000 {
        black_box(cg.next_sample());
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("subsynth_plain", |b| {
        b.iter_batched(
            || subsynth_plain(48000),
            |mut cg| subsynth_plain_x(&mut cg),
            BatchSize::SmallInput,
        )
    });

    c.bench_function("subsynth_with_containers", |b| {
        b.iter_batched(
            || subsynth_with_containers(48000),
            |mut cg| subsynth_with_containers_x(&mut cg),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
