use std::ops::Deref;

pub trait Wave {
    fn next_sample(&mut self) -> f64;
    fn sample_rate(&self) -> u32 {
        44100
    }
}

#[derive(Clone)]
pub struct WaveGenerator<T> {
    pub source: T,
}

impl<T> WaveGenerator<T> {
    pub fn new(source: T) -> Self {
        Self { source }
    }
}

impl<W> Wave for WaveGenerator<W>
where
    W: Wave,
{
    fn next_sample(&mut self) -> f64 {
        self.source.next_sample()
    }
}

impl<T> From<T> for WaveGenerator<T> {
    fn from(source: T) -> Self {
        Self::new(source)
    }
}

impl<T> Deref for WaveGenerator<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

impl<W> Iterator for WaveGenerator<W>
where
    W: Wave,
{
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.source.next_sample())
    }
}
