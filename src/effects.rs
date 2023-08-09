use std::collections::VecDeque;

use crate::{
    make_partial,
    partial_wave::{PartialWave, PartialWaveBuilder},
    wave::{Wave, WaveGenerator},
};

#[derive(Clone)]
pub struct Lowpass<T> {
    a1: f64,
    a2: f64,
    a3: f64,
    b1: f64,
    b2: f64,
    in_buffer: VecDeque<f64>,
    out_buffer: VecDeque<f64>,
    input: T,
}

impl<T> Lowpass<T> {
    pub fn new(f: f64, r: f64, input: T) -> WaveGenerator<Self> {
        let c = 1.0 / (std::f64::consts::PI * f / 44100.0);

        let a1 = 1.0 / (1.0 + r * c + c * c);
        let a2 = 2.0 * a1;
        let a3 = a1;
        let b1 = 2.0 * (1.0 - c * c) * a1;
        let b2 = (1.0 - r * c + c * c) * a1;

        let in_buffer = VecDeque::with_capacity(2);
        let out_buffer = VecDeque::with_capacity(2);

        Self {
            a1,
            a2,
            a3,
            b1,
            b2,
            in_buffer,
            out_buffer,
            input,
        }
        .into()
    }
}

impl<W> Wave for Lowpass<W>
where
    W: Wave,
{
    fn next_sample(&mut self) -> f64 {
        let in0 = self.input.next_sample();
        let in1 = *self.in_buffer.get(0).unwrap_or(&0.0);
        let in2 = self.in_buffer.pop_back().unwrap_or(0.0);
        self.in_buffer.push_front(in0);

        let out1 = *self.out_buffer.get(0).unwrap_or(&0.0);
        let out2 = self.out_buffer.pop_back().unwrap_or(0.0);

        let out = self.a1 * in0 + self.a2 * in1 + self.a3 * in2 - self.b1 * out1 - self.b2 * out2;

        self.out_buffer.push_front(out);
        out * 4.0
    }
}

make_partial!(
    PartialLowpass {
        f: f64,
        r: f64
    } => Lowpass
);

pub fn lowpass(f: f64, r: f64) -> PartialWaveBuilder<PartialLowpass> {
    PartialLowpass::new(f, r)
}
