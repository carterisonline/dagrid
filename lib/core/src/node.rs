use std::{borrow::Cow, f64::consts, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::*;

newtype!([cc, dd, o, e, Serialize, Deserialize] pub Sample = f64);

#[typetag::serde(tag = "type")]
pub trait Node: Debug + Send {
    fn get_ident(&self) -> &str;
    fn get_input_labels(&self) -> &[Cow<'_, str>];
    fn process(&self, inputs: &[Sample], phase: u64, sample_rate: u32) -> Sample;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Empty;

#[typetag::serde]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerInput(pub [Cow<'static, str>; 1]);

#[typetag::serde]
impl Node for ContainerInput {
    fn get_ident(&self) -> &str {
        "ContainerInput"
    }

    fn get_input_labels(&self) -> &[Cow<'static, str>] {
        &self.0
    }

    fn process(&self, inputs: &[Sample], _phase: u64, _sample_rate: u32) -> Sample {
        inputs[0]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerOutput(pub [Cow<'static, str>; 1]);

#[typetag::serde]
impl Node for ContainerOutput {
    fn get_ident(&self) -> &str {
        "ContainerOutput"
    }

    fn get_input_labels(&self) -> &[Cow<'static, str>] {
        &self.0
    }

    fn process(&self, inputs: &[Sample], _phase: u64, _sample_rate: u32) -> Sample {
        inputs[0]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sine;

#[typetag::serde]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Add;

#[typetag::serde]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Mul;

#[typetag::serde]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Inv;

#[typetag::serde]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Const(pub Sample);

#[typetag::serde]
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
