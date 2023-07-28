use std::f64::consts::TAU;

use crate::{
    oscillator::PartialOscillator,
    partial_wave::PartialWaveBuilder,
    variable::{VariableHandle, VariableSetter},
    wave::{Wave, WaveGenerator},
};

mod constant;
pub mod misc;

pub use constant::{Constant, VariableConstant};

pub fn constant<T: Into<f64>>(value: T) -> WaveGenerator<Constant> {
    Constant::new(value)
}

pub fn silence() -> WaveGenerator<Constant> {
    constant(0.0)
}

pub fn var<T: Into<f64>>(value: T) -> (WaveGenerator<VariableConstant>, impl VariableSetter<f64>) {
    VariableConstant::new(value)
}

pub fn var_dyn<T: Into<f64>>(value: T) -> (WaveGenerator<VariableConstant>, VariableHandle<f64>) {
    VariableConstant::new_dynamic(value)
}

pub fn sine<W: Wave>() -> PartialWaveBuilder<W, PartialOscillator> {
    PartialOscillator::new(|phase| (phase * TAU).sin()).into()
}

pub fn square() -> PartialOscillator {
    PartialOscillator::new(|phase| if phase < 0.5 { -1.0 } else { 1.0 })
}

pub fn saw() -> PartialOscillator {
    PartialOscillator::new(|phase| phase)
}

pub fn triangle() -> PartialOscillator {
    PartialOscillator::new(|phase| {
        if phase < 0.5 {
            phase * 4.0 - 1.0
        } else {
            3.0 - phase * 4.0
        }
    })
}
