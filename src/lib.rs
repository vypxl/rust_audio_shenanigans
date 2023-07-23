use std::sync::mpsc;

use cpal::{FromSample, Sample};
use wave_generator::WaveSource;

pub mod wave_generator;
pub mod waves;

pub struct WaveStreamer {
    wave_generator: Box<dyn WaveSource + Send>,
    wave_generator_update_channel: Option<mpsc::Receiver<Box<dyn WaveSource + Send>>>,
    sample_rate: u32,
    sample_rate_update_channel: Option<mpsc::Receiver<u32>>,
}

impl WaveStreamer {
    pub fn new(
        wave_generator: Box<dyn WaveSource + Send>,
        wave_generator_update_channel: Option<mpsc::Receiver<Box<dyn WaveSource + Send>>>,
        sample_rate: u32,
        sample_rate_update_channel: Option<mpsc::Receiver<u32>>,
    ) -> Self {
        Self {
            wave_generator,
            wave_generator_update_channel,
            sample_rate,
            sample_rate_update_channel,
        }
    }

    pub fn generate<T>(&mut self, buffer: &mut [T])
    where
        T: Sample + FromSample<f64>,
    {
        if let Some(chan) = &mut self.wave_generator_update_channel {
            if let Ok(wave_generator) = chan.try_recv() {
                self.wave_generator = wave_generator;
            }
        }

        if let Some(chan) = &mut self.sample_rate_update_channel {
            if let Ok(sample_rate) = chan.try_recv() {
                self.sample_rate = sample_rate;
            }
        }

        for sample in buffer.iter_mut() {
            *sample = Sample::from_sample(self.wave_generator.next_sample(self.sample_rate));
        }
    }
}
