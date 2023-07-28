use std::ops::Deref;

pub trait Wave {
    fn next_sample(&mut self) -> f64;
    fn sample_rate(&self) -> u32 {
        44100
    }
}

#[derive(Clone)]
pub struct WaveGenerator<T>
where
    T: Wave,
{
    pub source: T,
}

impl<T: Wave> WaveGenerator<T> {
    pub fn new(source: T) -> Self {
        Self { source }
    }
}

impl<T: Wave> Wave for WaveGenerator<T> {
    fn next_sample(&mut self) -> f64 {
        self.source.next_sample()
    }
}

impl<T: Wave> From<T> for WaveGenerator<T> {
    fn from(source: T) -> Self {
        Self::new(source)
    }
}

impl<T: Wave> Deref for WaveGenerator<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

impl<T: Wave> Iterator for WaveGenerator<T> {
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.source.next_sample())
    }
}
