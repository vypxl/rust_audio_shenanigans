use crate::{
    partial_wave::{PartialWave, PartialWaveBuilder},
    wave::{Wave, WaveGenerator},
};

type WaveFn = fn(f64) -> f64;

#[derive(Clone)]
pub struct Oscillator<T> {
    wave_fn: WaveFn,
    phase: f64,
    frequency: T,
}

impl<W> Oscillator<W>
where
    W: Wave,
{
    pub fn new(wave_fn: fn(f64) -> f64, frequency: W) -> WaveGenerator<Self> {
        Self {
            wave_fn,
            phase: 0.0,
            frequency,
        }
        .into()
    }
}

impl<W> Wave for Oscillator<W>
where
    W: Wave,
{
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
    pub fn new(wave_fn: WaveFn) -> PartialWaveBuilder<Self> {
        Self { wave_fn }.into()
    }
}

impl PartialWave for PartialOscillator {
    type Target<W: Wave> = Oscillator<W>;

    fn build<W>(self, src: W) -> WaveGenerator<Self::Target<W>>
    where
        W: Wave,
    {
        Oscillator::new(self.wave_fn, src)
    }
}
