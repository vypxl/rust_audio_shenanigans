use std::collections::HashMap;

use crate::{
    make_partial,
    partial_wave::{PartialWave, PartialWaveBuilder},
    wave::{Wave, WaveGenerator},
};

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

#[derive(Clone)]
pub struct Pass<W> {
    input: W,
}

impl<W> Pass<W> {
    pub fn new(input: W) -> WaveGenerator<Self> {
        Self { input }.into()
    }
}

impl<W: Wave> Wave for Pass<W> {
    #[inline]
    fn next_sample(&mut self) -> f64 {
        self.input.next_sample()
    }
}

make_partial!(PartialPass {} => Pass);
