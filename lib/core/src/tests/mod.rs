#![allow(unused_imports)]

use std::str::FromStr;
use std::{fs::File, io::Write, path::PathBuf};

use crate::assert_glicol_ref_eq;
use crate::container::*;
use crate::control::ControlGraph;
use crate::node::*;

mod glicol;

#[allow(dead_code)]
fn record_graph(test_name: &str, cg: &ControlGraph) {
    let graphs_path = PathBuf::from_str("src/tests/graphs").unwrap();

    std::fs::create_dir_all(&graphs_path).unwrap();
    let mut graph_file = File::create(graphs_path.join(format!("{test_name}.dot"))).unwrap();
    graph_file
        .write_all(crate::vis::visualize_graph(cg).as_bytes())
        .unwrap();
}

#[test]
fn subsynth_plain() {
    let mut cg = glicol::std_cg();

    let s1 = cg.connect_const_new(440.0, Sine);
    let s2 = cg.connect_const_new(220.0, Sine);
    let mulneg1 = cg.connect_const_new(-1.0, Mul);
    cg.connect(s2, mulneg1, 1);
    let add = cg.connect_many_new(&[mulneg1, s1], Add);
    let mulhalf = cg.connect_const_new(0.5, Mul);
    cg.connect(add, mulhalf, 1);
    cg.connect_existing_aout(mulhalf);

    record_graph("subsynth_plain", &cg);

    assert_glicol_ref_eq!(
        within epsilon * 26:
        &mut cg * 256 == "~s1: sin 440\n~s2: sin 220\no: ~s2 >> mul -1 >> add ~s1 >> mul 0.5"
    );
}

#[test]
fn subsynth_using_containers() {
    let mut cg = glicol::std_cg();

    // Create 440hz and 220hz oscillators
    let sine_osc_1 = cg.connect_const_new(440.0, Sine);
    let sine_osc_2 = cg.connect_const_new(220.0, Sine);

    // Subtract oscillator 1 from oscillator 2
    let (sub_in, sub_out) = cg.insert_container(Sub);
    cg.connect_ex_ex(sine_osc_1, sub_in[0]);
    cg.connect_ex_ex(sine_osc_2, sub_in[1]);

    // Audio out must be in range (-1 < x < 1)
    // divide by 2 to avoid exceeding that
    let (div_in, div_out) = cg.insert_container(Div);
    cg.connect_ex_ex(sub_out[0], div_in[0]);
    cg.connect_const_ex(2.0, div_in[1]);

    // Connect product to audio output
    cg.connect_existing_aout(div_out[0]);

    record_graph("subsynth_using_containers", &cg);

    assert_glicol_ref_eq!(
        within epsilon * 26:
        &mut cg * 256 == "~s1: sin 440\n~s2: sin 220\no: ~s2 >> mul -1 >> add ~s1 >> mul 0.5"
    );
}
