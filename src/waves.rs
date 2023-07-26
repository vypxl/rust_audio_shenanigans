use std::{
    f64::consts::PI,
    sync::{
        atomic::{self, AtomicU32},
        Arc,
    },
};

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

pub struct Sine {
    state: GeneratorState,
    pub pitch: Arc<AtomicU32>,
}

impl Sine {
    pub fn new() -> WaveGenerator<Self> {
        Self {
            state: GeneratorState { phase: 0.0 },
            pitch: Arc::new(440.into()),
        }
        .into()
    }
}

impl WaveSource for Sine {
    fn next_sample(&mut self) -> f64 {
        let increase = (self.pitch.load(atomic::Ordering::Relaxed) as f64 * 2.0 * PI)
            / self.sample_rate() as f64;
        self.state.phase += increase;
        self.state.phase %= 2.0 * PI;
        self.state.phase.sin()
    }
}

pub struct Square {
    state: GeneratorState,
    pub pitch: Arc<AtomicU32>,
}

impl Square {
    pub fn new() -> WaveGenerator<Self> {
        Self {
            state: GeneratorState { phase: 0.0 },
            pitch: Arc::new(440.into()),
        }
        .into()
    }
}

impl WaveSource for Square {
    fn next_sample(&mut self) -> f64 {
        let increase =
            self.pitch.load(atomic::Ordering::Relaxed) as f64 / self.sample_rate() as f64;
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

pub fn sine() -> WaveGenerator<Sine> {
    Sine::new()
}

pub fn square() -> WaveGenerator<Square> {
    Square::new()
}
