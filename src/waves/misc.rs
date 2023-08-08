use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Sub};

use crate::wave::{Wave, WaveGenerator};
use crate::waves::Constant;

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

#[derive(Clone)]
pub struct MixWaveSource<W, V> {
    mix: fn(f64, f64) -> f64,
    left: W,
    right: V,
}

impl<W, V> Wave for MixWaveSource<W, V>
where
    W: Wave,
    V: Wave,
{
    fn next_sample(&mut self) -> f64 {
        (self.mix)(self.left.next_sample(), self.right.next_sample())
    }
}

macro_rules! generator_op {
    ($trait_name:ident, $trait_fun:ident, $fun:expr) => {
        impl<W, V> $trait_name<WaveGenerator<V>> for WaveGenerator<W>
        where
            W: Wave,
            V: Wave,
        {
            type Output = WaveGenerator<MixWaveSource<W, V>>;
            fn $trait_fun(self, other: WaveGenerator<V>) -> Self::Output {
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
generator_op!(Sub, sub, |a, b| a - b);
generator_op!(Mul, mul, |a, b| a * b);
generator_op!(Div, div, |a, b| a / b);

macro_rules! generator_op_const {
    ($trait_name:ident, $trait_fun:ident, $fun:expr) => {
        impl<W, N> $trait_name<N> for WaveGenerator<W>
        where
            W: Wave,
            N: Into<f64>,
        {
            type Output = WaveGenerator<MixWaveSource<W, Constant>>;
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
generator_op_const!(Sub, sub, |a, b| a - b);
generator_op_const!(Mul, mul, |a, b| a * b);
generator_op_const!(Div, div, |a, b| a / b);

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
