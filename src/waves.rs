use std::{
    f64::consts::PI,
    sync::{
        atomic::{self, AtomicU32},
        Arc,
    },
};

use crate::{
    variable::VariableUpdater,
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
        return self.value;
    }
}

pub struct VariableConstant {
    pub value: Variable<f64>,
}

impl VariableConstant {
    pub fn new<T: Into<f64>>(value: T) -> (WaveGenerator<Self>, VariableUpdater<f64>) {
        let (var, updater) = Variable::new(value.into());
        (Self { value: var }.into(), updater)
    }
}

impl WaveSource for VariableConstant {
    fn next_sample(&mut self) -> f64 {
        return *self.value.update();
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
        return self.state.phase.sin();
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
        return if self.state.phase < 0.5 { 1.0 } else { -1.0 };
    }
}

fn sine() -> WaveGenerator<Sine> {
    return Sine::new();
}

fn square() -> WaveGenerator<Square> {
    return Square::new();
}
