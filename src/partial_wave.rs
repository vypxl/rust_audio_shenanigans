use crate::wave::WaveGenerator;
use crate::Wave;
use std::ops::Shr;

pub trait PartialWave {
    type Target<T: Wave>: Wave;
    fn build<T: Wave>(self, src: T) -> WaveGenerator<Self::Target<T>>;
}

#[derive(Clone)]
pub struct PartialWaveBuilder<T> {
    partial: T,
}

impl<T: PartialWave> From<T> for PartialWaveBuilder<T> {
    fn from(partial: T) -> Self {
        Self { partial }
    }
}

impl<T: PartialWave> PartialWave for PartialWaveBuilder<T> {
    type Target<W: Wave> = T::Target<W>;
    fn build<W: Wave>(self, src: W) -> WaveGenerator<T::Target<W>> {
        self.partial.build(src)
    }
}

impl<W: Wave, T: PartialWave> Shr<T> for WaveGenerator<W> {
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

impl<T: PartialWave, S: PartialWave> PartialWave for PartialWaveChain<T, S> {
    type Target<W: Wave> = S::Target<T::Target<W>>;
    fn build<W: Wave>(self, src: W) -> WaveGenerator<Self::Target<W>> {
        self.dst.build(self.src.build(src).source)
    }
}

impl<T: PartialWave, S: PartialWave> Shr<S> for PartialWaveBuilder<T> {
    type Output = PartialWaveChain<T, S>;
    fn shr(self, dest: S) -> Self::Output {
        PartialWaveChain {
            src: self.partial,
            dst: dest,
        }
    }
}
