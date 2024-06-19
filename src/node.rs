use std::{borrow::Cow, f64::consts, fmt::Debug};

use crate::*;

newtype!([cc, dd, o, e] pub Sample = f64);

pub trait Node: Debug {
    fn get_ident(&self) -> &str;
    fn get_input_labels(&self) -> &[Cow<'_, str>];
    fn process(&self, inputs: &[Sample], phase: u64, sample_rate: u32) -> Sample;
}

#[derive(Debug)]
pub struct Empty;
impl Node for Empty {
    fn get_ident(&self) -> &str {
        "Empty"
    }

    fn get_input_labels(&self) -> &[Cow<'_, str>] {
        &[]
    }

    fn process(&self, _inputs: &[Sample], _phase: u64, _sample_rate: u32) -> Sample {
        0.0.into()
    }
}

#[derive(Debug)]
pub struct ContainerInput<'a>(pub [Cow<'a, str>; 1]);
impl<'a> Node for ContainerInput<'a> {
    fn get_ident(&self) -> &str {
        "ContainerInput"
    }

    fn get_input_labels(&self) -> &[Cow<'a, str>] {
        &self.0
    }

    fn process(&self, inputs: &[Sample], _phase: u64, _sample_rate: u32) -> Sample {
        inputs[0]
    }
}

#[derive(Debug)]
pub struct ContainerOutput<'a>(pub [Cow<'a, str>; 1]);
impl<'a> Node for ContainerOutput<'a> {
    fn get_ident(&self) -> &str {
        "ContainerOutput"
    }

    fn get_input_labels(&self) -> &[Cow<'a, str>] {
        &self.0
    }

    fn process(&self, inputs: &[Sample], _phase: u64, _sample_rate: u32) -> Sample {
        inputs[0]
    }
}

#[derive(Debug)]
pub struct Sine;
impl Node for Sine {
    fn get_ident(&self) -> &str {
        "Sine"
    }

    fn get_input_labels(&self) -> &[Cow<'static, str>] {
        &[Cow::Borrowed("Frequency")]
    }

    fn process(&self, inputs: &[Sample], phase: u64, sample_rate: u32) -> Sample {
        let phase_delta = *inputs[0] / (sample_rate as f64);
        let sine = ((phase as f64) * phase_delta * consts::TAU).sin();

        Sample(sine)
    }
}

#[derive(Debug)]
pub struct Add;
impl Node for Add {
    fn get_ident(&self) -> &str {
        "Add"
    }

    fn get_input_labels<'a>(&self) -> &[Cow<'static, str>] {
        &[Cow::Borrowed("LHS"), Cow::Borrowed("RHS")]
    }

    fn process(&self, inputs: &[Sample], _phase: u64, _sample_rate: u32) -> Sample {
        Sample(*inputs[0] + *inputs[1])
    }
}

#[derive(Debug)]
pub struct Mul;
impl Node for Mul {
    fn get_ident(&self) -> &str {
        "Multiply"
    }

    fn get_input_labels<'a>(&self) -> &[Cow<'static, str>] {
        &[Cow::Borrowed("LHS"), Cow::Borrowed("RHS")]
    }

    fn process(&self, inputs: &[Sample], _phase: u64, _sample_rate: u32) -> Sample {
        Sample(*inputs[0] * *inputs[1])
    }
}

#[derive(Debug)]
pub struct Inv;
impl Node for Inv {
    fn get_ident(&self) -> &str {
        "Inverse"
    }

    fn get_input_labels<'a>(&self) -> &[Cow<'static, str>] {
        &[Cow::Borrowed("Input")]
    }

    fn process(&self, inputs: &[Sample], _phase: u64, _sample_rate: u32) -> Sample {
        Sample(1.0 / *inputs[0])
    }
}

#[derive(Debug)]
pub struct Const(pub Sample);
impl Node for Const {
    fn get_ident(&self) -> &str {
        "Constant"
    }

    fn get_input_labels<'a>(&self) -> &[Cow<'static, str>] {
        &[]
    }

    fn process(&self, _inputs: &[Sample], _phase: u64, _sample_rate: u32) -> Sample {
        self.0
    }
}

impl From<Sample> for Const {
    fn from(value: Sample) -> Self {
        Self(value)
    }
}

pub fn c(sample: f64) -> Const {
    Const(sample.into())
}
