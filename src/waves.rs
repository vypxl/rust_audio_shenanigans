use std::f64::consts::TAU;

use crate::{
    variable::{VariableSetter, VariableUpdater},
    wave_generator::{WaveGenerator, WaveSource},
    Variable,
};

struct GeneratorState {
    pub phase: f64,
}

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

pub struct Sine<T: WaveSource> {
    state: GeneratorState,
    pub pitch: T,
}

impl<T: WaveSource> Sine<T> {
    pub fn new(pitch_source: T) -> WaveGenerator<Self> {
        Self {
            state: GeneratorState { phase: 0.0 },
            pitch: pitch_source,
        }
        .into()
    }
}

impl<T: WaveSource> WaveSource for Sine<T> {
    fn next_sample(&mut self) -> f64 {
        let increase = (self.pitch.next_sample() * TAU) / self.sample_rate() as f64;
        self.state.phase += increase;
        self.state.phase %= TAU;
        self.state.phase.sin()
    }
}

pub struct Square<T: WaveSource> {
    state: GeneratorState,
    pub pitch: T,
}

impl<T: WaveSource> Square<T> {
    pub fn new(pitch_source: T) -> WaveGenerator<Self> {
        Self {
            state: GeneratorState { phase: 0.0 },
            pitch: pitch_source,
        }
        .into()
    }
}

impl<T: WaveSource> WaveSource for Square<T> {
    fn next_sample(&mut self) -> f64 {
        let increase = self.pitch.next_sample() / self.sample_rate() as f64;
        self.state.phase += increase;
        self.state.phase %= 1.0;
        if self.state.phase < 0.5 {
            1.0
        } else {
            -1.0
        }
    }
}

pub fn constant<T: Into<f64>>(value: T) -> WaveGenerator<Constant> {
    Constant::new(value)
}

pub fn var<T: Into<f64>>(value: T) -> (WaveGenerator<VariableConstant>, impl VariableSetter<f64>) {
    VariableConstant::new(value)
}

pub fn var_dyn<T: Into<f64>>(
    value: T,
) -> (WaveGenerator<VariableConstant>, impl VariableUpdater<f64>) {
    VariableConstant::new_dynamic(value)
}

pub fn sine<T: WaveSource>(pitch_source: T) -> WaveGenerator<Sine<T>> {
    Sine::new(pitch_source)
}

pub fn square<T: WaveSource>(pitch_source: T) -> WaveGenerator<Square<T>> {
    Square::new(pitch_source)
}
