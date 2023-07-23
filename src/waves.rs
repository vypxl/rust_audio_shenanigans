use std::{
    f64::consts::PI,
    sync::{
        atomic::{self, AtomicU32},
        Arc,
    },
};

use crate::wave_generator::{WaveGenerator, WaveSource};

struct GeneratorState {
    pub phase: f64,
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
    fn next_sample(&mut self, sample_rate: u32) -> f64 {
        let increase =
            (self.pitch.load(atomic::Ordering::Relaxed) as f64 * 2.0 * PI) / sample_rate as f64;
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
    fn next_sample(&mut self, sample_rate: u32) -> f64 {
        let increase = self.pitch.load(atomic::Ordering::Relaxed) as f64 / sample_rate as f64;
        self.state.phase += increase;
        self.state.phase %= 1.0;
        return if self.state.phase < 0.5 { 1.0 } else { -1.0 };
    }
}
