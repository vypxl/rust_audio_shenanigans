use crate::{
    partial_wave::PartialWave,
    wave::{Wave, WaveGenerator},
};

type WaveFn = fn(f64) -> f64;

#[derive(Clone)]
pub struct Oscillator<T: Wave> {
    wave_fn: WaveFn,
    phase: f64,
    frequency: T,
}

impl<T: Wave> Oscillator<T> {
    pub fn new(wave_fn: fn(f64) -> f64, frequency: T) -> WaveGenerator<Self> {
        Self {
            wave_fn,
            phase: 0.0,
            frequency,
        }
        .into()
    }
}

impl<T: Wave> Wave for Oscillator<T> {
    fn next_sample(&mut self) -> f64 {
        let increase = self.frequency.next_sample() / self.sample_rate() as f64;
        self.phase += increase;
        self.phase %= 1.0;

        (self.wave_fn)(self.phase)
    }
}

#[derive(Clone)]
pub struct PartialOscillator {
    wave_fn: WaveFn,
}

impl PartialOscillator {
    pub fn new(wave_fn: WaveFn) -> Self {
        Self { wave_fn }
    }
}

impl<W: Wave> PartialWave<W> for PartialOscillator {
    type Target = Oscillator<W>;

    fn build(self, src: W) -> WaveGenerator<Self::Target> {
        Oscillator::new(self.wave_fn, src)
    }
}
