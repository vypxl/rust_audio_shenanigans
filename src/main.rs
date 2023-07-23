use std::{
    error::Error,
    sync::{atomic, mpsc},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use rust_audio::{wave_generator::*, waves, *};

fn main() -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no output device available")?;
    let mut supported_configs_range = device.supported_output_configs()?;
    let supported_config = supported_configs_range
        .next()
        .ok_or("no supported config?!")?;

    let config = supported_config.with_max_sample_rate().config();

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let generator_a = waves::Square::new();
    let remote_pitch = generator_a.pitch.clone();
    let mut pitch = remote_pitch.load(atomic::Ordering::Relaxed) as f64;

    let generator_b = waves::Sine::new();

    let generator = generator_a + generator_b;

    let (tx, rx) = mpsc::channel();

    let mut streamer = WaveStreamer::new(Box::new(generator), Some(rx), config.sample_rate.0, None);

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _| streamer.generate(data),
            err_fn,
            None,
        )
        .unwrap();

    stream.play()?;

    let half_tone_up = 2.0f64.powf(1.0 / 12.0);
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        pitch *= half_tone_up;
        remote_pitch.store(pitch as u32, atomic::Ordering::Relaxed);
    }
}
