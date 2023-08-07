use crate::wave::{WaveBox, WaveGenerator};
use std::ops::Shr;

pub type PartialWaveBox = Box<dyn PartialWave>;

pub trait PartialWave {
    fn build(&self, src: WaveBox) -> WaveGenerator;
}

pub struct PartialWaveBuilder {
    partial: Box<dyn PartialWave>,
}

impl PartialWaveBuilder {
    pub fn new<T: PartialWave + 'static>(partial: T) -> Self {
        Self {
            partial: Box::new(partial),
        }
    }
}

impl PartialWave for PartialWaveBuilder {
    fn build(&self, src: WaveBox) -> WaveGenerator {
        self.partial.build(src)
    }
}

impl<T: PartialWave> Shr<T> for WaveGenerator {
    type Output = WaveGenerator;
    fn shr(self, dest: T) -> Self::Output {
        dest.build(self.0)
    }
}

pub struct PartialWaveChain {
    src: Box<dyn PartialWave>,
    dst: Box<dyn PartialWave>,
}

impl PartialWave for PartialWaveChain {
    fn build(&self, src: WaveBox) -> WaveGenerator {
        self.dst.build(self.src.build(src).0)
    }
}

impl<S: PartialWave + 'static> Shr<S> for PartialWaveBuilder {
    type Output = PartialWaveChain;
    fn shr(self, dest: S) -> Self::Output {
        PartialWaveChain {
            src: self.partial,
            dst: Box::new(dest),
        }
    }
}
