use petgraph::graph::NodeIndex;

use crate::control::ControlGraph;
use crate::node::*;

pub trait Container {
    fn get_ident(&self) -> &str;
    fn get_input_labels(&self) -> &[&str];
    fn get_output_labels(&self) -> &[&str];
    fn construct(&self, inputs: &[NodeIndex], outputs: &[NodeIndex], cg: &mut ControlGraph);
}

pub struct Sub;
impl Container for Sub {
    fn get_ident(&self) -> &str {
        "Subtract"
    }

    fn get_input_labels(&self) -> &[&str] {
        &["LHS", "RHS"]
    }

    fn get_output_labels(&self) -> &[&str] {
        &["Difference"]
    }

    fn construct(&self, inputs: &[NodeIndex], outputs: &[NodeIndex], cg: &mut ControlGraph) {
        let mul_neg = cg.connect_const_new(-1.0, Mul);
        cg.connect(inputs[1], mul_neg, 1);

        let add = cg.connect_many_new(&[inputs[0], mul_neg], Add);

        cg.connect_ex_ex(add, outputs[0]);
    }
}

pub struct Div;
impl Container for Div {
    fn get_ident(&self) -> &str {
        "Divide"
    }

    fn get_input_labels(&self) -> &[&str] {
        &["Dividend", "Divisor"]
    }

    fn get_output_labels(&self) -> &[&str] {
        &["Quotient"]
    }

    fn construct(&self, inputs: &[NodeIndex], outputs: &[NodeIndex], cg: &mut ControlGraph) {
        let inv = cg.connect_ex_new(inputs[1], Inv);
        let mul = cg.connect_many_new(&[inputs[0], inv], Mul);

        cg.connect_ex_ex(mul, outputs[0]);
    }
}
