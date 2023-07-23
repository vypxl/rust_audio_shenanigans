use std::ops::{Add, Deref};

pub trait WaveSource {
    fn next_sample(&mut self, sample_rate: u32) -> f64;
}

pub struct WaveGenerator<T>
where
    T: WaveSource,
{
    pub source: T,
}

impl<T: WaveSource> WaveGenerator<T> {
    pub fn new(source: T) -> Self {
        Self { source }
    }
}

impl<T: WaveSource> WaveSource for WaveGenerator<T> {
    fn next_sample(&mut self, sample_rate: u32) -> f64 {
        self.source.next_sample(sample_rate)
    }
}

impl<T: WaveSource> From<T> for WaveGenerator<T> {
    fn from(source: T) -> Self {
        Self::new(source)
    }
}

impl<T: WaveSource> Deref for WaveGenerator<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

pub struct AddWaveSource<T, U>
where
    T: WaveSource,
    U: WaveSource,
{
    left: T,
    right: U,
}

impl<T, U> WaveSource for AddWaveSource<T, U>
where
    T: WaveSource,
    U: WaveSource,
{
    fn next_sample(&mut self, sample_rate: u32) -> f64 {
        self.left.next_sample(sample_rate) + self.right.next_sample(sample_rate)
    }
}

impl<T, U> Add<WaveGenerator<U>> for WaveGenerator<T>
where
    T: WaveSource,
    U: WaveSource,
{
    type Output = WaveGenerator<AddWaveSource<T, U>>;
    fn add(self, other: WaveGenerator<U>) -> Self::Output {
        WaveGenerator {
            source: AddWaveSource {
                left: self.source,
                right: other.source,
            },
        }
    }
}
