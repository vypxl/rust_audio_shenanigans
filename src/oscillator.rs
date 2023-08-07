use crate::{
    partial_wave::{PartialWave, PartialWaveBuilder},
    wave::{Wave, WaveBox, WaveGenerator},
};

type WaveFn = fn(f64) -> f64;

pub struct Oscillator {
    wave_fn: WaveFn,
    phase: f64,
    frequency: WaveBox,
}

impl Oscillator {
    pub fn make(wave_fn: fn(f64) -> f64, frequency: WaveBox) -> WaveGenerator {
        WaveGenerator::new(Self {
            wave_fn,
            phase: 0.0,
            frequency,
        })
    }
}

impl Wave for Oscillator {
    fn next_sample(&mut self) -> f64 {
        let increase = self.frequency.next_sample() / self.sample_rate() as f64;
        self.phase += increase;
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }

        (self.wave_fn)(self.phase)
    }
}

#[derive(Clone)]
pub struct PartialOscillator {
    wave_fn: WaveFn,
}

impl PartialOscillator {
    pub fn make(wave_fn: WaveFn) -> PartialWaveBuilder {
        PartialWaveBuilder::new(Self { wave_fn })
    }
}

impl PartialWave for PartialOscillator {
    fn build(&self, src: WaveBox) -> WaveGenerator {
        Oscillator::make(self.wave_fn, src)
    }
}
