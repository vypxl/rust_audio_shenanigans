use std::ops::{Add, Deref, Mul};

use crate::waves::Constant;

pub trait WaveSource {
    fn next_sample(&mut self) -> f64;
    fn sample_rate(&self) -> u32 {
        44100
    }
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
    fn next_sample(&mut self) -> f64 {
        self.source.next_sample()
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

impl<T: WaveSource> Iterator for WaveGenerator<T> {
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.source.next_sample())
    }
}

pub struct IteratorWaveSource<T>
where
    T: Iterator<Item = f64>,
{
    iter: T,
}

impl<T: Iterator<Item = f64>> WaveSource for IteratorWaveSource<T> {
    fn next_sample(&mut self) -> f64 {
        self.iter.next().unwrap_or(0.0)
    }
}

impl<T: Iterator<Item = f64>> From<T> for WaveGenerator<IteratorWaveSource<T>> {
    fn from(iter: T) -> Self {
        Self::new(IteratorWaveSource { iter })
    }
}

pub struct MixWaveSource<T, U>
where
    T: WaveSource,
    U: WaveSource,
{
    mix: fn(f64, f64) -> f64,
    left: T,
    right: U,
}

impl<T, U> WaveSource for MixWaveSource<T, U>
where
    T: WaveSource,
    U: WaveSource,
{
    fn next_sample(&mut self) -> f64 {
        (self.mix)(self.left.next_sample(), self.right.next_sample())
    }
}

macro_rules! generator_op {
    ($trait_name:ident, $trait_fun:ident, $fun:expr) => {
        impl<T, U> $trait_name<WaveGenerator<U>> for WaveGenerator<T>
        where
            T: WaveSource,
            U: WaveSource,
        {
            type Output = WaveGenerator<MixWaveSource<T, U>>;
            fn $trait_fun(self, other: WaveGenerator<U>) -> Self::Output {
                WaveGenerator {
                    source: MixWaveSource {
                        mix: $fun,
                        left: self.source,
                        right: other.source,
                    },
                }
            }
        }
    };
}

generator_op!(Add, add, |a, b| a + b);
generator_op!(Mul, mul, |a, b| a * b);

macro_rules! generator_op_const {
    ($trait_name:ident, $trait_fun:ident, $fun:expr) => {
        impl<T, N> $trait_name<N> for WaveGenerator<T>
        where
            T: WaveSource,
            N: Into<f64>,
        {
            type Output = WaveGenerator<MixWaveSource<T, Constant>>;
            fn $trait_fun(self, other: N) -> Self::Output {
                MixWaveSource {
                    mix: $fun,
                    left: self.source,
                    right: Constant {
                        value: other.into(),
                    },
                }
                .into()
            }
        }
    };
}

generator_op_const!(Add, add, |a, b| a + b);
generator_op_const!(Mul, mul, |a, b| a * b);
