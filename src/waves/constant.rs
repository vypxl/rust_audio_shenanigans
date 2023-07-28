use crate::{
    variable::{VariableHandle, VariableSetter},
    wave::{Wave, WaveGenerator},
    Variable,
};

#[derive(Clone)]
pub struct Constant {
    pub value: f64,
}

impl Constant {
    pub fn new<T: Into<f64>>(value: T) -> WaveGenerator<Self> {
        Self {
            value: value.into(),
        }
        .into()
    }
}

impl Wave for Constant {
    fn next_sample(&mut self) -> f64 {
        self.value
    }
}

pub struct VariableConstant {
    pub value: Variable<f64>,
}

impl VariableConstant {
    pub fn new<T: Into<f64>>(value: T) -> (WaveGenerator<Self>, impl VariableSetter<f64>) {
        let (var, updater) = Variable::new(value.into());
        (Self { value: var }.into(), updater)
    }

    pub fn new_dynamic<T: Into<f64>>(value: T) -> (WaveGenerator<Self>, VariableHandle<f64>) {
        let (var, updater) = Variable::new_dynamic(value.into());
        (Self { value: var }.into(), updater)
    }
}

impl Wave for VariableConstant {
    fn next_sample(&mut self) -> f64 {
        *self.value.update()
    }
}
