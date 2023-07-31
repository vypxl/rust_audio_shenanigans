use crate::{
    partial_wave::{PartialWave, PartialWaveBuilder},
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
pub struct PartialOscillator<W> {
    _w_marker: std::marker::PhantomData<W>,
    wave_fn: WaveFn,
}

impl<W: Wave> PartialOscillator<W> {
    pub fn new(wave_fn: WaveFn) -> PartialWaveBuilder<W, Self> {
        Self {
            _w_marker: std::marker::PhantomData,
            wave_fn,
        }
        .into()
    }
}

impl<W: Wave> PartialWave<W> for PartialOscillator<W> {
    type Target = Oscillator<W>;

    fn build(self, src: W) -> WaveGenerator<Self::Target> {
        Oscillator::new(self.wave_fn, src)
    }
}
