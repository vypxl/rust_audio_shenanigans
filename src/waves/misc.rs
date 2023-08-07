use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Sub};

use crate::wave::{Wave, WaveBox, WaveGenerator};
use crate::waves::Constant;

#[derive(Clone)]
pub struct IteratorWaveSource<T>
where
    T: Iterator<Item = f64>,
{
    iter: T,
}

impl<T: Iterator<Item = f64> + Clone> Wave for IteratorWaveSource<T> {
    fn next_sample(&mut self) -> f64 {
        self.iter.next().unwrap_or(0.0)
    }
}

impl<T: Iterator<Item = f64> + Clone + 'static + Send> From<T> for WaveGenerator {
    fn from(iter: T) -> Self {
        Self::new(IteratorWaveSource { iter })
    }
}

pub struct MixWaveSource {
    mix: fn(f64, f64) -> f64,
    left: WaveBox,
    right: WaveBox,
}

impl Wave for MixWaveSource {
    fn next_sample(&mut self) -> f64 {
        (self.mix)(self.left.next_sample(), self.right.next_sample())
    }
}

macro_rules! generator_op {
    ($trait_name:ident, $trait_fun:ident, $fun:expr) => {
        impl $trait_name<WaveGenerator> for WaveGenerator {
            type Output = WaveGenerator;
            fn $trait_fun(self, other: WaveGenerator) -> Self::Output {
                WaveGenerator::new(MixWaveSource {
                    mix: $fun,
                    left: self.0,
                    right: other.0,
                })
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
        impl<N> $trait_name<N> for WaveGenerator
        where
            N: Into<f64>,
        {
            type Output = WaveGenerator;
            fn $trait_fun(self, other: N) -> Self::Output {
                WaveGenerator::new(MixWaveSource {
                    mix: $fun,
                    left: self.0,
                    right: Box::new(Constant {
                        value: other.into(),
                    }),
                })
            }
        }
    };
}

generator_op_const!(Add, add, |a, b| a + b);
generator_op_const!(Sub, sub, |a, b| a - b);
generator_op_const!(Mul, mul, |a, b| a * b);
generator_op_const!(Div, div, |a, b| a / b);

impl<T: Wave> Wave for Vec<T> {
    fn next_sample(&mut self) -> f64 {
        self.iter_mut().map(|w| w.next_sample()).sum()
    }
}

impl<T: Wave, K> Wave for HashMap<K, T> {
    fn next_sample(&mut self) -> f64 {
        self.values_mut().map(|w| w.next_sample()).sum()
    }
}
