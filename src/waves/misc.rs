use std::ops::{Add, Div, Mul, Sub};

use crate::wave::{Wave, WaveGenerator};
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

impl<T: Iterator<Item = f64> + Clone> From<T> for WaveGenerator<IteratorWaveSource<T>> {
    fn from(iter: T) -> Self {
        Self::new(IteratorWaveSource { iter })
    }
}

#[derive(Clone)]
pub struct MixWaveSource<T, U>
where
    T: Wave,
    U: Wave,
{
    mix: fn(f64, f64) -> f64,
    left: T,
    right: U,
}

impl<T, U> Wave for MixWaveSource<T, U>
where
    T: Wave,
    U: Wave,
{
    fn next_sample(&mut self) -> f64 {
        (self.mix)(self.left.next_sample(), self.right.next_sample())
    }
}

macro_rules! generator_op {
    ($trait_name:ident, $trait_fun:ident, $fun:expr) => {
        impl<T, U> $trait_name<WaveGenerator<U>> for WaveGenerator<T>
        where
            T: Wave,
            U: Wave,
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
generator_op!(Sub, sub, |a, b| a - b);
generator_op!(Mul, mul, |a, b| a * b);
generator_op!(Div, div, |a, b| a / b);

macro_rules! generator_op_const {
    ($trait_name:ident, $trait_fun:ident, $fun:expr) => {
        impl<T, N> $trait_name<N> for WaveGenerator<T>
        where
            T: Wave,
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
generator_op_const!(Sub, sub, |a, b| a - b);
generator_op_const!(Mul, mul, |a, b| a * b);
generator_op_const!(Div, div, |a, b| a / b);

impl<T: Wave> Wave for Vec<T> {
    fn next_sample(&mut self) -> f64 {
        self.iter_mut().map(|w| w.next_sample()).sum()
    }
}
