#![allow(unused_imports)]

use std::str::FromStr;
use std::{fs::File, io::Write, path::PathBuf};

use crate::container::*;
use crate::control::ControlGraph;
use crate::node::*;
use crate::presets::preset;
use crate::{assert_glicol_ref_eq, presets};

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
    let mut cg = preset(44100, presets::subsynth_plain);

    record_graph("subsynth_plain", &cg);

    assert_glicol_ref_eq!(
        within epsilon * 26:
        &mut cg * 256 == "~s1: sin 440\n~s2: sin 220\no: ~s2 >> mul -1 >> add ~s1 >> mul 0.5"
    );
}

#[test]
fn subsynth_with_containers() {
    let mut cg = preset(44100, presets::subsynth_with_containers);

    record_graph("subsynth_with_containers", &cg);

    assert_glicol_ref_eq!(
        within epsilon * 26:
        &mut cg * 256 == "~s1: sin 440\n~s2: sin 220\no: ~s2 >> mul -1 >> add ~s1 >> mul 0.5"
    );
}
