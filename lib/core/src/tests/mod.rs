#![allow(unused_imports)]

use std::str::FromStr;
use std::{fs::File, io::Write, path::PathBuf};

use petgraph::graph::{EdgeIndex, NodeIndex};

use crate::container::*;
use crate::control::ControlGraph;
use crate::node::*;
use crate::presets::preset;
use crate::{assert_glicol_ref_eq, presets};

mod common;

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

#[test]
fn subsynth_with_containers_save_load() {
    let mut cg1 = preset(44100, presets::subsynth_with_containers);

    let mut cg2 = ControlGraph::load(44100, &cg1.save().unwrap()).unwrap();

    let s1 = common::cg_samples::<256>(&mut cg1);
    let s2 = common::cg_samples::<256>(&mut cg2);

    if s1 != s2 {
        panic!(
            "{}",
            common::nonmatching_report::<256>(&s2, &s1, &common::eq_matches::<256>(&s2, &s1, 1))
        );
    }
}

#[test]
fn subsynth_plain_patch() {
    let mut cg = preset(44100, presets::subsynth_plain);

    cg.remove(NodeIndex::new(1));
    cg.connect_const_ex(330.0, NodeIndex::new(2));

    record_graph("subsynth_plain_patch", &cg);

    assert_glicol_ref_eq!(
        within epsilon * 70:
        &mut cg * 256 == "~s1: sin 330\n~s2: sin 220\no: ~s2 >> mul -1 >> add ~s1 >> mul 0.5"
    );
}

#[test]
fn subsynth_plain_disconnect() {
    let mut cg = preset(44100, presets::subsynth_plain);

    cg.connect_const_ex(110.0, NodeIndex::new(4));
    cg.disconnect(EdgeIndex::new(1));

    record_graph("subsynth_plain_disconnect", &cg);

    dbg!(&cg);

    assert_glicol_ref_eq!(
        within epsilon * 70:
        &mut cg * 256 == "~s1: sin 440\n~s2: sin 110\no: ~s2 >> mul -1 >> add ~s1 >> mul 0.5"
    );
}
