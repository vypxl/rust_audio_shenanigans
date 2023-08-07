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
    pub fn make<T: Into<f64>>(value: T) -> WaveGenerator {
        WaveGenerator::new(Self {
            value: value.into(),
        })
    }
}

impl Wave for Constant {
    fn next_sample(&mut self) -> f64 {
        self.value
    }
}

#[derive(Clone)]
pub struct VariableConstant {
    pub value: Variable<f64>,
}

impl VariableConstant {
    pub fn make<T: Into<f64>>(value: T) -> (WaveGenerator, impl VariableSetter<f64>) {
        let (var, updater) = Variable::new(value.into());
        (WaveGenerator::new(Self { value: var }), updater)
    }

    pub fn new_dynamic<T: Into<f64>>(value: T) -> (WaveGenerator, VariableHandle<f64>) {
        let (var, updater) = Variable::new_dynamic(value.into());
        (WaveGenerator::new(Self { value: var }), updater)
    }
}

impl Wave for VariableConstant {
    fn next_sample(&mut self) -> f64 {
        *self.value.update()
    }
}
