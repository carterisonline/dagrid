#![allow(unused_imports)]

use std::borrow::Cow;
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
fn subsynth_plain_multiout() {
    let mut cg = preset(44100, presets::subsynth_plain_multiout);

    record_graph("subsynth_plain_multiout", &cg);

    assert_glicol_ref_eq!(
        // feeding a sin into another sin is weird, so dg deviates a bit. still matches the curve
        within epsilon * 140_000:
        &mut cg * 256 == "~s1: sin 220\n~s2: sin 220\n~s3: sin ~s2\no: ~s3 >> add ~s1 >> mul 0.5"
    );
}

#[test]
fn subsynth_with_containers() {
    let mut cg = preset(44100, presets::subsynth_with_containers);

    record_graph("subsynth_with_containers", &cg);

    assert_glicol_ref_eq!(
        within epsilon * 26:
        &mut cg * 2 == "~s1: sin 440\n~s2: sin 220\no: ~s2 >> mul -1 >> add ~s1 >> mul 0.5"
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

    cg.disconnect(EdgeIndex::new(1));
    cg.connect_const_ex(110.0, NodeIndex::new(4));

    record_graph("subsynth_plain_disconnect", &cg);

    dbg!(&cg);

    assert_glicol_ref_eq!(
        within epsilon * 70:
        &mut cg * 256 == "~s1: sin 440\n~s2: sin 110\no: ~s2 >> mul -1 >> add ~s1 >> mul 0.5"
    );
}

#[test]
fn subsynth_with_containers_disconnect() {
    let mut cg = preset(44100, presets::subsynth_with_containers);

    assert_glicol_ref_eq!(
        within epsilon * 26:
        &mut cg * 256 == "~s1: sin 440\n~s2: sin 220\no: ~s2 >> mul -1 >> add ~s1 >> mul 0.5"
    );

    cg.disconnect(EdgeIndex::new(14));
    cg.connect_const_ex(4.0, NodeIndex::new(12));
    cg.reset_phase();

    record_graph("subsynth_with_containers_disconnect", &cg);

    assert_glicol_ref_eq!(
        within epsilon * 26:
        &mut cg * 256 == "~s1: sin 440\n~s2: sin 220\no: ~s2 >> mul -1 >> add ~s1 >> mul 0.25"
    );
}

#[test]
fn disconnect() {
    let mut cg = preset(44100, |cg| {
        let c = cg.connect_const_new(1.0, ContainerOutput([Cow::Borrowed("disconnect")]));
        cg.connect_ex_aout(c);
    });

    record_graph("disconnect_before", &cg);

    assert_eq!(cg.next_sample().l(), 1.0);

    cg.disconnect(EdgeIndex::new(0));
    let c = cg.insert(c(2.0));
    cg.connect_ex_ex(c, NodeIndex::new(2));

    record_graph("disconnect_after", &cg);

    assert_eq!(cg.next_sample().l(), 2.0);
}
