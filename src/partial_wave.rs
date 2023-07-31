use crate::wave::WaveGenerator;
use crate::Wave;
use std::ops::Shr;

pub trait PartialWave<W: Wave> {
    type Target: Wave;
    fn build(self, src: W) -> WaveGenerator<Self::Target>;
}

#[derive(Clone)]
pub struct PartialWaveBuilder<W: Wave, T: PartialWave<W>> {
    _w_marker: std::marker::PhantomData<W>,
    partial: T,
}

impl<W: Wave, T: PartialWave<W>> From<T> for PartialWaveBuilder<W, T> {
    fn from(partial: T) -> Self {
        Self {
            _w_marker: std::marker::PhantomData,
            partial,
        }
    }
}

impl<W: Wave, T: PartialWave<W>> PartialWave<W> for PartialWaveBuilder<W, T> {
    type Target = T::Target;
    fn build(self, src: W) -> WaveGenerator<T::Target> {
        self.partial.build(src)
    }
}

impl<W: Wave, T: PartialWave<W>> Shr<T> for WaveGenerator<W> {
    type Output = WaveGenerator<T::Target>;
    fn shr(self, dest: T) -> Self::Output {
        dest.build(self.source)
    }
}

#[derive(Clone)]
pub struct PartialWaveChain<W: Wave, T: PartialWave<W>, S: PartialWave<T::Target>> {
    _w_marker: std::marker::PhantomData<W>,
    src: T,
    dst: S,
}

impl<W: Wave, T: PartialWave<W>, S: PartialWave<T::Target>> PartialWave<W>
    for PartialWaveChain<W, T, S>
{
    type Target = S::Target;
    fn build(self, src: W) -> WaveGenerator<Self::Target> {
        self.dst.build(self.src.build(src).source)
    }
}

impl<W: Wave, T: PartialWave<W>, S: PartialWave<T::Target>> Shr<S> for PartialWaveBuilder<W, T> {
    type Output = PartialWaveChain<W, T, S>;
    fn shr(self, dest: S) -> Self::Output {
        PartialWaveChain {
            _w_marker: std::marker::PhantomData,
            src: self.partial,
            dst: dest,
        }
    }
}
