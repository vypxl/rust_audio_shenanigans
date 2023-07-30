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
