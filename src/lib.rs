#![feature(array_chunks)]
#![feature(type_alias_impl_trait)]

use std::sync::{Arc, Mutex};

use cpal::{FromSample, Sample};
use wave::Wave;

pub mod effects;
pub mod instrument;
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
    wave_generator: GeneratorArc,
    sample_rate: Variable<u32>,
}

impl WaveStreamer {
    pub fn new(wave_generator: impl Wave + Send + 'static, sample_rate: u32) -> Self {
        Self::new_raw(Arc::new(Mutex::new(Box::new(wave_generator))), sample_rate)
    }

    pub fn new_raw(wave_generator: GeneratorArc, sample_rate: u32) -> Self {
        Self {
            wave_generator,
            sample_rate: Variable::new(sample_rate).0,
        }
    }

    pub fn new_var(wave_generator: impl Wave + Send + 'static, sample_rate: Variable<u32>) -> Self {
        Self {
            wave_generator: Arc::new(Mutex::new(Box::new(wave_generator))),
            sample_rate,
        }
    }

    pub fn generate<T>(&mut self, buffer: &mut [T])
    where
        T: Sample + FromSample<f64>,
    {
        let mut gen = self.wave_generator.lock().unwrap();
        self.sample_rate.update();

        for [sample_l, sample_r] in buffer.array_chunks_mut() {
            *sample_r = Sample::from_sample(gen.next_sample());
            *sample_l = *sample_r;
        }
    }
}
