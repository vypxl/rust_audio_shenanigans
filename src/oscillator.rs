use std::f64::consts::TAU;

use crate::{
    partial_wave::{PartialWave, PartialWaveBuilder},
    wave::{Wave, WaveGenerator},
};

pub trait WaveFn {
    fn process(&mut self, phase: f64) -> f64;
}

macro_rules! make_wave_fn {
    ($name:ident, $phase:ident => $expr:expr) => {
        #[derive(Clone)]
        pub struct $name;
        impl WaveFn for $name {
            #[inline]
            fn process(&mut self, $phase: f64) -> f64 {
                $expr
            }
        }
    };
}

make_wave_fn!(Sine, phase => (phase * TAU).sin());
make_wave_fn!(Square, phase => if phase < 0.5 { -1.0 } else { 1.0 });
make_wave_fn!(Sawtooth, phase => phase);
make_wave_fn!(Triangle, phase => if phase < 0.5 { phase * 4.0 - 1.0 } else { 3.0 - phase * 4.0 });

#[derive(Clone)]
pub struct Oscillator<T, F> {
    wave_fn: F,
    phase: f64,
    input: T,
}

impl<W, F> Oscillator<W, F>
where
    W: Wave,
    F: WaveFn,
{
    pub fn new(wave_fn: F, input: W) -> WaveGenerator<Self> {
        Self {
            wave_fn,
            phase: 0.0,
            input,
        }
        .into()
    }
}

impl<W, F> Wave for Oscillator<W, F>
where
    W: Wave,
    F: WaveFn,
{
    #[inline]
    fn next_sample(&mut self) -> f64 {
        let increase = self.input.next_sample() / self.sample_rate() as f64;
        self.phase += increase;
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }

        self.wave_fn.process(self.phase)
    }
}

#[derive(Clone)]
pub struct PartialOscillator<F> {
    wave_fn: F,
}

impl<F> PartialOscillator<F>
where
    F: WaveFn,
{
    pub fn new(wave_fn: F) -> PartialWaveBuilder<Self> {
        Self { wave_fn }.into()
    }
}

impl<F> PartialWave for PartialOscillator<F>
where
    F: WaveFn,
{
    type Target<W: Wave> = Oscillator<W, F>;
    fn build<W>(self, input: W) -> WaveGenerator<Self::Target<W>>
    where
        W: Wave,
    {
        Oscillator::new(self.wave_fn, input)
    }
}
