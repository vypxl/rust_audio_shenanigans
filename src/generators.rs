use cpal::{FromSample, Sample};
use std::{
    f64::consts::PI,
    sync::{
        atomic::{self, AtomicU32},
        Arc,
    },
};

pub trait WaveGenerator {
    fn next_sample(&mut self) -> f64;

    fn generate<T>(&mut self, buffer: &mut [T])
    where
        T: Sample + FromSample<f64>,
    {
        for sample in buffer.iter_mut() {
            *sample = Sample::from_sample(self.next_sample());
        }
    }
}

struct GeneratorState {
    pub sample_rate: u32,
    pub phase: f64,
}

pub struct SineWaveGenerator {
    state: GeneratorState,
    pub pitch: Arc<AtomicU32>,
}

impl SineWaveGenerator {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            state: GeneratorState {
                sample_rate,
                phase: 0.0,
            },
            pitch: Arc::new(440.into()),
        }
    }
}

impl WaveGenerator for SineWaveGenerator {
    fn next_sample(&mut self) -> f64 {
        let increase = (self.pitch.load(atomic::Ordering::Relaxed) as f64 * 2.0 * PI)
            / self.state.sample_rate as f64;
        self.state.phase += increase;
        self.state.phase %= 2.0 * PI;
        return self.state.phase.sin();
    }
}

pub struct SquareWaveGenerator {
    state: GeneratorState,
    pub pitch: Arc<AtomicU32>,
}

impl SquareWaveGenerator {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            state: GeneratorState {
                sample_rate,
                phase: 0.0,
            },
            pitch: Arc::new(440.into()),
        }
    }
}

impl WaveGenerator for SquareWaveGenerator {
    fn next_sample(&mut self) -> f64 {
        let increase =
            self.pitch.load(atomic::Ordering::Relaxed) as f64 / self.state.sample_rate as f64;
        self.state.phase += increase;
        self.state.phase %= 1.0;
        return if self.state.phase < 0.5 { 1.0 } else { -1.0 };
    }
}
