use std::{f64::consts::TAU, ops::Shr};

use crate::{
    variable::{VariableSetter, VariableUpdater},
    wave_generator::{Wave, WaveGenerator},
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

    pub fn new_dynamic<T: Into<f64>>(value: T) -> (WaveGenerator<Self>, impl VariableUpdater<f64>) {
        let (var, updater) = Variable::new_dynamic(value.into());
        (Self { value: var }.into(), updater)
    }
}

impl Wave for VariableConstant {
    fn next_sample(&mut self) -> f64 {
        *self.value.update()
    }
}

type WaveFn = fn(f64) -> f64;

#[derive(Clone)]
pub struct Oscillator<T: Wave> {
    wave_fn: WaveFn,
    phase: f64,
    frequency: T,
}

impl<T: Wave> Oscillator<T> {
    pub fn new(wave_fn: fn(f64) -> f64, frequency: T) -> WaveGenerator<Self> {
        Self {
            wave_fn,
            phase: 0.0,
            frequency,
        }
        .into()
    }
}

impl<T: Wave> Wave for Oscillator<T> {
    fn next_sample(&mut self) -> f64 {
        let increase = self.frequency.next_sample() / self.sample_rate() as f64;
        self.phase += increase;
        self.phase %= 1.0;

        (self.wave_fn)(self.phase)
    }
}

pub struct PartialOscillator {
    wave_fn: WaveFn,
}

pub trait PartialWave<W: Wave> {
    type Target: Wave;
    fn build(self, src: W) -> WaveGenerator<Self::Target>;
}

impl<W: Wave> PartialWave<W> for PartialOscillator {
    type Target = Oscillator<W>;

    fn build(self, src: W) -> WaveGenerator<Self::Target> {
        Oscillator::new(self.wave_fn, src)
    }
}

pub struct PartialWaveBuilder<W: Wave, T: PartialWave<W>> {
    _w_marker: std::marker::PhantomData<W>,
    partial: T,
}

impl<W: Wave, T: PartialWave<W>> From<T> for PartialWaveBuilder<W, T> {
    fn from(partial: T) -> Self {
        Self {
            _w_marker: std::marker::PhantomData,
            partial,
        }
    }
}

impl<W: Wave, T: PartialWave<W>> PartialWave<W> for PartialWaveBuilder<W, T> {
    type Target = T::Target;
    fn build(self, src: W) -> WaveGenerator<T::Target> {
        self.partial.build(src)
    }
}

impl<W: Wave, T: PartialWave<W>> Shr<T> for WaveGenerator<W> {
    type Output = WaveGenerator<T::Target>;
    fn shr(self, dest: T) -> Self::Output {
        dest.build(self.source)
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

pub fn sine<W: Wave>() -> PartialWaveBuilder<W, PartialOscillator> {
    PartialOscillator {
        wave_fn: |phase| (phase * TAU).sin(),
    }
    .into()
}

pub fn square() -> PartialOscillator {
    PartialOscillator {
        wave_fn: |phase| if phase < 0.5 { 0.0 } else { 1.0 },
    }
}

pub fn saw() -> PartialOscillator {
    PartialOscillator {
        wave_fn: |phase| phase,
    }
}

pub fn triangle() -> PartialOscillator {
    PartialOscillator {
        wave_fn: |phase| {
            if phase < 0.5 {
                phase * 4.0 - 1.0
            } else {
                3.0 - phase * 4.0
            }
        },
    }
}
