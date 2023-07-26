use std::f64::consts::TAU;

use crate::{
    variable::{VariableSetter, VariableUpdater},
    wave_generator::{WaveGenerator, WaveSource},
    Variable,
};

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

impl WaveSource for Constant {
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

    pub fn new_dynamic<T: Into<f64>>(value: T) -> (WaveGenerator<Self>, impl VariableUpdater<f64>) {
        let (var, updater) = Variable::new_dynamic(value.into());
        (Self { value: var }.into(), updater)
    }
}

impl WaveSource for VariableConstant {
    fn next_sample(&mut self) -> f64 {
        *self.value.update()
    }
}

pub struct Oscillator<T: WaveSource> {
    phase: f64,
    pitch: T,
    process: fn(f64) -> f64,
}

impl<T: WaveSource> Oscillator<T> {
    pub fn new(process: fn(f64) -> f64, pitch_source: T) -> WaveGenerator<Self> {
        Self {
            phase: 0.0,
            pitch: pitch_source,
            process,
        }
        .into()
    }
}

    }
}

impl<T: WaveSource> WaveSource for Oscillator<T> {
    fn next_sample(&mut self) -> f64 {
        let increase = self.pitch.next_sample() / self.sample_rate() as f64;
        self.phase += increase;
        self.phase %= 1.0;

        (self.process)(self.phase)
    }
}

pub fn constant<T: Into<f64>>(value: T) -> WaveGenerator<Constant> {
    Constant::new(value)
}

pub fn silence() -> WaveGenerator<Constant> {
    constant(0.0)
}

pub fn var<T: Into<f64>>(value: T) -> (WaveGenerator<VariableConstant>, impl VariableSetter<f64>) {
    VariableConstant::new(value)
}

pub fn var_dyn<T: Into<f64>>(
    value: T,
) -> (WaveGenerator<VariableConstant>, impl VariableUpdater<f64>) {
    VariableConstant::new_dynamic(value)
}

pub fn sine<T: WaveSource>(pitch_source: T) -> WaveGenerator<Oscillator<T>> {
    Oscillator::new(|phase| (phase * TAU).sin(), pitch_source)
}

pub fn square<T: WaveSource>(pitch_source: T) -> WaveGenerator<Oscillator<T>> {
    Oscillator::new(|phase| if phase < 0.5 { 0.0 } else { 1.0 }, pitch_source)
}

pub fn sawtooth<T: WaveSource>(pitch_source: T) -> WaveGenerator<Oscillator<T>> {
    Oscillator::new(|phase| phase, pitch_source)
}

pub fn triangle<T: WaveSource>(pitch_source: T) -> WaveGenerator<Oscillator<T>> {
    Oscillator::new(
        |phase| {
            if phase < 0.5 {
                phase * 4.0 - 1.0
            } else {
                3.0 - phase * 4.0
            }
        },
        pitch_source,
    )
}
