use crate::{
    make_partial,
    partial_wave::{PartialWave, PartialWaveBuilder},
    wave::{Wave, WaveGenerator},
};

type WaveFn = fn(f64) -> f64;

#[derive(Clone)]
pub struct Oscillator<T> {
    wave_fn: WaveFn,
    phase: f64,
    input: T,
}

impl<W> Oscillator<W>
where
    W: Wave,
{
    pub fn new(wave_fn: fn(f64) -> f64, input: W) -> WaveGenerator<Self> {
        Self {
            wave_fn,
            phase: 0.0,
            input,
        }
        .into()
    }
}

impl<W> Wave for Oscillator<W>
where
    W: Wave,
{
    fn next_sample(&mut self) -> f64 {
        let increase = self.input.next_sample() / self.sample_rate() as f64;
        self.phase += increase;
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }

        (self.wave_fn)(self.phase)
    }
}

make_partial!(
    PartialOscillator {
        wave: WaveFn
    } => Oscillator
);
