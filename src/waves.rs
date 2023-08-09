use crate::{
    oscillator::{PartialOscillator, Sawtooth, Sine, Square, Triangle},
    partial_wave::PartialWaveBuilder,
    variable::{VariableHandle, VariableSetter},
    wave::WaveGenerator,
};

mod adsr;
mod constant;
pub mod misc;
mod mix;

pub use adsr::{ADSREvent, Trigger as ADSRTrigger, ADSR};
pub use constant::{Constant, VariableConstant};
pub use mix::WaveMixer;

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

pub fn sine() -> PartialWaveBuilder<PartialOscillator<Sine>> {
    PartialOscillator::new(Sine)
}

pub fn square() -> PartialWaveBuilder<PartialOscillator<Square>> {
    PartialOscillator::new(Square)
}

pub fn sawtooth() -> PartialWaveBuilder<PartialOscillator<Sawtooth>> {
    PartialOscillator::new(Sawtooth)
}

pub fn triangle() -> PartialWaveBuilder<PartialOscillator<Triangle>> {
    PartialOscillator::new(Triangle)
}
