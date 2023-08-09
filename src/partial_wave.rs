use crate::wave::WaveGenerator;
use crate::Wave;
use std::ops::Shr;

pub trait PartialWave {
    type Target<W: Wave>: Wave;

    fn build<W>(self, src: W) -> WaveGenerator<Self::Target<W>>
    where
        W: Wave;
}

#[derive(Clone)]
pub struct PartialWaveBuilder<T> {
    partial: T,
}

impl<T> From<T> for PartialWaveBuilder<T>
where
    T: PartialWave,
{
    fn from(partial: T) -> Self {
        Self { partial }
    }
}

impl<T> PartialWave for PartialWaveBuilder<T>
where
    T: PartialWave,
{
    type Target<W: Wave> = T::Target<W>;

    fn build<W>(self, src: W) -> WaveGenerator<T::Target<W>>
    where
        W: Wave,
    {
        self.partial.build(src)
    }
}

impl<W, T> Shr<T> for WaveGenerator<W>
where
    W: Wave,
    T: PartialWave,
{
    type Output = WaveGenerator<T::Target<W>>;
    fn shr(self, dest: T) -> Self::Output {
        dest.build(self.source)
    }
}

#[derive(Clone)]
pub struct PartialWaveChain<T, S> {
    src: T,
    dst: S,
}

impl<T, S> PartialWave for PartialWaveChain<T, S>
where
    T: PartialWave,
    S: PartialWave,
{
    type Target<W: Wave> = S::Target<T::Target<W>>;
    fn build<W>(self, src: W) -> WaveGenerator<Self::Target<W>>
    where
        W: Wave,
    {
        self.dst.build(self.src.build(src).source)
    }
}

impl<T, S> Shr<S> for PartialWaveBuilder<T>
where
    T: PartialWave,
    S: PartialWave,
{
    type Output = PartialWaveChain<T, S>;
    fn shr(self, dest: S) -> Self::Output {
        PartialWaveChain {
            src: self.partial,
            dst: dest,
        }
    }
}

/// This macro can be used to automatically generate a struct implementing partial wave instead of
/// defining it manually. It takes a struct name, a list of fields and their types, and the name of
/// the struct to generate. It only works when the target struct has a `new` method that takes the
/// listed parameters in the same order, followed by an `input` parameter.
///
/// Example usage:
/// ```
/// use rust_audio_shenanigans::make_partial;
/// use rust_audio_shenanigans::partial_wave::{PartialWave, PartialWaveBuilder};
/// use rust_audio_shenanigans::wave::{Wave, WaveGenerator};
/// use rust_audio_shenanigans::waves::{constant, sine};
///
/// pub struct MyWave<T> { mul: f64, input: T }
///
/// impl<W> MyWave<W> where W: Wave {
///     fn new(mul: f64, input: W) -> WaveGenerator<Self> { Self { mul, input }.into() }
/// }
///
/// impl<W: Wave> Wave for MyWave<W> {
///     fn next_sample(&mut self) -> f64 { self.input.next_sample() * self.mul }
/// }
///
/// make_partial!(PartialMyWave { mul: f64 } => MyWave);
///
/// fn use_partial() {
///     let partial = PartialMyWave::new(2.0);
///     let wave = constant(440.0) >> sine();
///
///     let result = wave >> partial; // Same as MyWave::new(2.0, wave)
/// }
/// ```
#[macro_export]
macro_rules! make_partial {
    ($partial_name:ident { $($field_name:ident: $field_type:ty),* } => $target:ident) => {
        #[derive(Clone)]
        pub struct $partial_name {
            $($field_name: $field_type,)*
        }

        impl $partial_name {
            pub fn new($($field_name: $field_type,)*) -> PartialWaveBuilder<Self> {
                Self { $($field_name,)* }.into()
            }
        }

        impl PartialWave for $partial_name {
            type Target<W: Wave> = $target<W>;

            fn build<W>(self, input: W) -> WaveGenerator<Self::Target<W>>
            where
                W: Wave,
            {
                $target::new($(self.$field_name,)* input)
            }
        }
    };
}
