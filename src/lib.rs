#![feature(array_chunks)]

use cpal::{FromSample, Sample};
use wave::Wave;

mod oscillator;
pub mod partial_wave;
mod variable;
pub mod wave;
pub mod waves;

pub use oscillator::Oscillator;
pub use variable::Variable;

type Generator = Box<dyn Wave + Send>;

pub struct WaveStreamer {
    wave_generator_l: Variable<Generator>,
    wave_generator_r: Variable<Generator>,
    sample_rate: Variable<u32>,
}

impl WaveStreamer {
    pub fn new(wave_generator_l: Generator, wave_generator_r: Generator, sample_rate: u32) -> Self {
        Self {
            wave_generator_l: Variable::new(wave_generator_l).0,
            wave_generator_r: Variable::new(wave_generator_r).0,
            sample_rate: Variable::new(sample_rate).0,
        }
    }

    pub fn new_var(
        wave_generator_l: Variable<Generator>,
        wave_generator_r: Variable<Generator>,
        sample_rate: Variable<u32>,
    ) -> Self {
        Self {
            wave_generator_l,
            wave_generator_r,
            sample_rate,
        }
    }

    pub fn generate<T>(&mut self, buffer: &mut [T])
    where
        T: Sample + FromSample<f64>,
    {
        self.wave_generator_l.update();
        self.wave_generator_r.update();
        self.sample_rate.update();

        for [sample_l, sample_r] in buffer.array_chunks_mut() {
            *sample_l = Sample::from_sample(self.wave_generator_l.next_sample());
            *sample_r = Sample::from_sample(self.wave_generator_r.next_sample());
        }
    }
}
