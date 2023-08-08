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
