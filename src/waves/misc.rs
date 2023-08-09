use std::collections::HashMap;

use crate::wave::{Wave, WaveGenerator};

#[derive(Clone)]
pub struct IteratorWaveSource<T> {
    iter: T,
}

impl<T: Iterator<Item = f64> + Clone> Wave for IteratorWaveSource<T> {
    fn next_sample(&mut self) -> f64 {
        self.iter.next().unwrap_or(0.0)
    }
}

impl<T: Iterator<Item = f64> + Clone> From<T> for WaveGenerator<IteratorWaveSource<T>> {
    fn from(iter: T) -> Self {
        Self::new(IteratorWaveSource { iter })
    }
}

impl<W: Wave> Wave for Vec<W> {
    fn next_sample(&mut self) -> f64 {
        self.iter_mut().map(|w| w.next_sample()).sum()
    }
}

impl<W: Wave, K> Wave for HashMap<K, W> {
    fn next_sample(&mut self) -> f64 {
        self.values_mut().map(|w| w.next_sample()).sum()
    }
}
