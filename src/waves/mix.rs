/// Implementation of WaveMixer, which allows mixing two waves together in various ways.
///
/// Implementation details:
/// For some reason, creating structs and implementing the simple mix methods on them is faster than
/// using closures. Probably because thanks to the generic argument that I can pass this way, the
/// compiler can inline the functions, which it doesn't do for closures in this case.
use crate::{
    partial_wave::{PartialWave, PartialWaveBuilder},
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
        #[derive(Clone)]
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

#[derive(Clone)]
pub struct PartialWaveMixer1<M, W, T> {
    mixer: M,
    left: W,
    right: T,
}

impl<M, W, T> PartialWaveMixer1<M, W, T>
where
    M: MixFn,
    W: Wave,
    T: PartialWave,
{
    pub fn new(mixer: M, left: W, right: T) -> Self {
        Self { mixer, left, right }
    }
}

impl<M, W, T> PartialWave for PartialWaveMixer1<M, W, T>
where
    M: MixFn + Clone + Send + Sync,
    W: Wave + Clone + Send + Sync,
    T: PartialWave,
{
    type Target<V: Wave + Clone + Send + Sync> = WaveMixer<M, W, WaveGenerator<T::Target<V>>>;
    fn build<V>(self, src: V) -> WaveGenerator<Self::Target<V>>
    where
        V: Wave + Clone + Send + Sync,
    {
        WaveMixer {
            mixer: self.mixer,
            left: self.left,
            right: self.right.build(src),
        }
        .into()
    }
}

#[derive(Clone)]
pub struct PartialWaveMixer2<M, T, U> {
    mixer: M,
    left: T,
    right: U,
}

impl<M, T, U> PartialWaveMixer2<M, T, U>
where
    M: MixFn,
    T: PartialWave,
    U: PartialWave,
{
    pub fn new(mixer: M, left: T, right: U) -> Self {
        Self { mixer, left, right }
    }
}

impl<M, T, U> PartialWave for PartialWaveMixer2<M, T, U>
where
    M: MixFn + Clone + Send + Sync,
    T: PartialWave,
    U: PartialWave,
{
    type Target<V: Wave + Clone + Send + Sync> =
        WaveMixer<M, WaveGenerator<T::Target<V>>, WaveGenerator<U::Target<V>>>;
    fn build<V>(self, src: V) -> WaveGenerator<Self::Target<V>>
    where
        V: Wave + Clone + Send + Sync,
    {
        WaveMixer {
            mixer: self.mixer,
            left: self.left.build(src.clone()),
            right: self.right.build(src),
        }
        .into()
    }
}

macro_rules! partial_op {
    ($trait_name:ident, $trait_fun:ident, $mixer:ident) => {
        impl<W, T> $trait_name<WaveGenerator<W>> for PartialWaveBuilder<T>
        where
            T: PartialWave,
            W: Wave + Clone + Send + Sync,
        {
            type Output = PartialWaveBuilder<PartialWaveMixer1<$mixer, WaveGenerator<W>, T>>;
            fn $trait_fun(self, other: WaveGenerator<W>) -> Self::Output {
                PartialWaveMixer1 {
                    mixer: $mixer,
                    left: other,
                    right: self.into_inner(),
                }
                .into()
            }
        }

        impl<W, T> $trait_name<PartialWaveBuilder<T>> for WaveGenerator<W>
        where
            T: PartialWave,
            W: Wave + Clone + Send + Sync,
        {
            type Output = PartialWaveBuilder<PartialWaveMixer1<$mixer, WaveGenerator<W>, T>>;
            fn $trait_fun(self, other: PartialWaveBuilder<T>) -> Self::Output {
                PartialWaveMixer1 {
                    mixer: $mixer,
                    left: self,
                    right: other.into_inner(),
                }
                .into()
            }
        }

        impl<T, U> $trait_name<PartialWaveBuilder<U>> for PartialWaveBuilder<T>
        where
            T: PartialWave,
            U: PartialWave,
        {
            type Output = PartialWaveBuilder<PartialWaveMixer2<$mixer, T, U>>;
            fn $trait_fun(self, other: PartialWaveBuilder<U>) -> Self::Output {
                PartialWaveMixer2 {
                    mixer: $mixer,
                    left: self.into_inner(),
                    right: other.into_inner(),
                }
                .into()
            }
        }
    };
}

partial_op!(Add, add, MixAdd);
partial_op!(Sub, sub, MixSub);
partial_op!(Mul, mul, MixMul);
partial_op!(Div, div, MixDiv);

macro_rules! partial_op_const {
    ($trait_name:ident, $trait_fun:ident, $mixer:ident) => {
        impl<T, N> $trait_name<N> for PartialWaveBuilder<T>
        where
            T: PartialWave,
            N: Into<f64>,
        {
            type Output = PartialWaveBuilder<PartialWaveMixer1<$mixer, Constant, T>>;
            fn $trait_fun(self, other: N) -> Self::Output {
                PartialWaveMixer1 {
                    mixer: $mixer,
                    left: Constant {
                        value: other.into(),
                    },
                    right: self.into_inner(),
                }
                .into()
            }
        }
    };
}

partial_op_const!(Add, add, MixAdd);
partial_op_const!(Sub, sub, MixSub);
partial_op_const!(Mul, mul, MixMul);
partial_op_const!(Div, div, MixDiv);
