#![feature(array_chunks)]

use std::sync::{Arc, Mutex};

use cpal::{FromSample, Sample};
use wave::Wave;

mod oscillator;
pub mod partial_wave;
mod variable;
pub mod wave;
pub mod waves;

pub use oscillator::Oscillator;
pub use variable::Variable;

type Generator = dyn Wave + Send;
type GeneratorArc = Arc<Mutex<Box<Generator>>>;

pub struct WaveStreamer {
    wave_generator_l: GeneratorArc,
    wave_generator_r: GeneratorArc,
    sample_rate: Variable<u32>,
}

impl WaveStreamer {
    pub fn new(
        wave_generator_l: impl Wave + Send + 'static,
        wave_generator_r: impl Wave + Send + 'static,
        sample_rate: u32,
    ) -> Self {
        Self {
            wave_generator_l: Arc::new(Mutex::new(Box::new(wave_generator_l))),
            wave_generator_r: Arc::new(Mutex::new(Box::new(wave_generator_r))),
            sample_rate: Variable::new(sample_rate).0,
        }
    }

    pub fn new_var(
        wave_generator_l: impl Wave + Send + 'static,
        wave_generator_r: impl Wave + Send + 'static,
        sample_rate: Variable<u32>,
    ) -> Self {
        Self {
            wave_generator_l: Arc::new(Mutex::new(Box::new(wave_generator_l))),
            wave_generator_r: Arc::new(Mutex::new(Box::new(wave_generator_r))),
            sample_rate,
        }
    }

    pub fn generate<T>(&mut self, buffer: &mut [T])
    where
        T: Sample + FromSample<f64>,
    {
        let mut gen_l = self.wave_generator_l.lock().unwrap();
        let mut gen_r = self.wave_generator_r.lock().unwrap();
        self.sample_rate.update();

        for [sample_l, sample_r] in buffer.array_chunks_mut() {
            *sample_l = Sample::from_sample(gen_l.next_sample());
            *sample_r = Sample::from_sample(gen_r.next_sample());
        }
    }
}
