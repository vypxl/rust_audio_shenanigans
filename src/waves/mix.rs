/// Implementation of WaveMixer, which allows mixing two waves together in various ways.
///
/// Implementation details:
/// For some reason, creating structs and implementing the simple mix methods on them is faster than
/// using closures. Probably because thanks to the generic argument that I can pass this way, the
/// compiler can inline the functions, which it doesn't do for closures in this case.
use crate::{
    wave::{Wave, WaveGenerator},
    waves::Constant,
};
use std::ops::{Add, Div, Mul, Sub};

pub trait MixFn {
    fn mix(&self, left: f64, right: f64) -> f64;
}

/// Create a struct implementing MixFn by combining `left` and `right` with the given expression.
macro_rules! make_mixer {
    ($name:ident, $left:ident $right:ident => $fun:expr) => {
        pub struct $name;
        impl MixFn for $name {
            #[inline]
            fn mix(&self, $left: f64, $right: f64) -> f64 {
                $fun
            }
        }
    };
}

make_mixer!(MixAdd, a b => a + b);
make_mixer!(MixSub, a b => a - b);
make_mixer!(MixMul, a b => a * b);
make_mixer!(MixDiv, a b => a / b);

#[derive(Clone)]
pub struct WaveMixer<M, W, V> {
    mixer: M,
    left: W,
    right: V,
}

impl<M, W, V> Wave for WaveMixer<M, W, V>
where
    M: MixFn,
    W: Wave,
    V: Wave,
{
    #[inline]
    fn next_sample(&mut self) -> f64 {
        self.mixer
            .mix(self.left.next_sample(), self.right.next_sample())
    }
}

macro_rules! generator_op {
    ($trait_name:ident, $trait_fun:ident, $mixer:ident) => {
        impl<W, V> $trait_name<WaveGenerator<V>> for WaveGenerator<W>
        where
            W: Wave,
            V: Wave,
        {
            type Output = WaveGenerator<WaveMixer<$mixer, W, V>>;
            fn $trait_fun(self, other: WaveGenerator<V>) -> Self::Output {
                WaveGenerator {
                    source: WaveMixer {
                        mixer: $mixer,
                        left: self.source,
                        right: other.source,
                    },
                }
            }
        }
    };
}

generator_op!(Add, add, MixAdd);
generator_op!(Sub, sub, MixSub);
generator_op!(Mul, mul, MixMul);
generator_op!(Div, div, MixDiv);

macro_rules! generator_op_const {
    ($trait_name:ident, $trait_fun:ident, $mixer:ident) => {
        impl<W, N> $trait_name<N> for WaveGenerator<W>
        where
            W: Wave,
            N: Into<f64>,
        {
            type Output = WaveGenerator<WaveMixer<$mixer, W, Constant>>;
            fn $trait_fun(self, other: N) -> Self::Output {
                WaveMixer {
                    mixer: $mixer,
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

generator_op_const!(Add, add, MixAdd);
generator_op_const!(Sub, sub, MixSub);
generator_op_const!(Mul, mul, MixMul);
generator_op_const!(Div, div, MixDiv);
