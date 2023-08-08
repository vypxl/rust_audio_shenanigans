use std::f64::consts::TAU;

use crate::{
    oscillator::PartialOscillator,
    partial_wave::PartialWaveBuilder,
    variable::{VariableHandle, VariableSetter},
    wave::WaveGenerator,
};

mod adsr;
mod constant;
pub mod misc;

pub use adsr::{ADSREvent, Trigger as ADSRTrigger, ADSR};
pub use constant::{Constant, VariableConstant};

pub fn constant<T>(value: T) -> WaveGenerator<Constant>
where
    T: Into<f64>,
{
    Constant::new(value)
}

pub fn silence() -> WaveGenerator<Constant> {
    constant(0.0)
}

pub fn var<T>(value: T) -> (WaveGenerator<VariableConstant>, impl VariableSetter<f64>)
where
    T: Into<f64>,
{
    VariableConstant::new(value)
}

pub fn var_dyn<T>(value: T) -> (WaveGenerator<VariableConstant>, VariableHandle<f64>)
where
    T: Into<f64>,
{
    VariableConstant::new_dynamic(value)
}

pub fn sine() -> PartialWaveBuilder<PartialOscillator> {
    PartialOscillator::new(|phase| (phase * TAU).sin())
}

pub fn square() -> PartialWaveBuilder<PartialOscillator> {
    PartialOscillator::new(|phase| if phase < 0.5 { -1.0 } else { 1.0 })
}

pub fn sawtooth() -> PartialWaveBuilder<PartialOscillator> {
    PartialOscillator::new(|phase| phase)
}

pub fn triangle() -> PartialWaveBuilder<PartialOscillator> {
    PartialOscillator::new(|phase| {
        if phase < 0.5 {
            phase * 4.0 - 1.0
        } else {
            3.0 - phase * 4.0
        }
    })
}
