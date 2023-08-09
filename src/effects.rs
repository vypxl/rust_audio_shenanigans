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
    in_buffer: [f64; 2],
    out_buffer: [f64; 2],
    offset: usize,
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

        let in_buffer = [0.0; 2];
        let out_buffer = [0.0; 2];

        Self {
            a1,
            a2,
            a3,
            b1,
            b2,
            in_buffer,
            out_buffer,
            offset: 0,
            input,
        }
        .into()
    }
}

impl<W> Wave for Lowpass<W>
where
    W: Wave,
{
    #[inline]
    fn next_sample(&mut self) -> f64 {
        let i1 = self.offset;
        let i2 = 1 - self.offset;

        let in0 = self.input.next_sample();
        let in1 = self.in_buffer[i1];
        let in2 = self.in_buffer[i2];
        self.in_buffer[i2] = in0;

        let out1 = self.out_buffer[i1];
        let out2 = self.out_buffer[i2];

        let out = self.a1 * in0 + self.a2 * in1 + self.a3 * in2 - self.b1 * out1 - self.b2 * out2;

        self.out_buffer[i2] = out;

        self.offset = i2;

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
