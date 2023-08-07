use std::ops::Deref;

pub trait Wave {
    fn next_sample(&mut self) -> f64;
    fn sample_rate(&self) -> u32 {
        44100
    }
}

// #[derive(Clone)]
// pub struct WaveGenerator
// {
//     pub source: T,
// }
pub type WaveBox = Box<dyn Wave + Send>;

pub struct WaveGenerator(pub WaveBox);

impl WaveGenerator {
    pub fn new<T: Wave + 'static + Send>(source: T) -> Self {
        Self(Box::new(source))
    }
}

impl Wave for WaveGenerator {
    fn next_sample(&mut self) -> f64 {
        self.0.next_sample()
    }
}

// impl<T: Wave> From<T> for WaveGenerator {
//     fn from(source: T) -> Self {
//         Self::new(source)
//     }
// }

impl Deref for WaveGenerator {
    type Target = WaveBox;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Iterator for WaveGenerator {
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.next_sample())
    }
}
