use std::{error::Error, sync::atomic};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use rust_audio::{generators::*, *};

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

    let mut generator = SquareWaveGenerator::new(config.sample_rate.0);
    let remote_pitch = generator.pitch.clone();
    let mut pitch = remote_pitch.load(atomic::Ordering::Relaxed) as f64;

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _| generator.generate(data),
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
