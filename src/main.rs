use std::error::Error;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use rust_audio_shenanigans::{wave_generator::*, waves, *};

fn setup_device() -> Result<(cpal::Device, cpal::StreamConfig), Box<dyn Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no output device available")?;
    let mut supported_configs_range = device.supported_output_configs()?;
    let supported_config = supported_configs_range
        .next()
        .ok_or("no supported config?!")?;

    Ok((
        device,
        cpal::StreamConfig {
            channels: 2,
            ..supported_config
                .with_sample_rate(cpal::SampleRate(44100))
                .config()
        },
    ))
}

fn setup_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut streamer: WaveStreamer,
) -> Result<cpal::Stream, Box<dyn Error>> {
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _| streamer.generate(data),
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn setup_streamer(sample_rate: u32) -> WaveStreamer {
    let generator_a = waves::square();

    let (mul, update_mul) = waves::var_dyn(1.0);
    let generator_b = waves::square() * mul;

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        update_mul(Box::new(|v| *v *= -1.0));
    });

    WaveStreamer::new(Box::new(generator_a), Box::new(generator_b), sample_rate)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (device, config) = setup_device()?;

    let streamer = setup_streamer(config.sample_rate.0);

    let stream = setup_stream(&device, &config, streamer)?;
    stream.play()?;

    std::thread::park();
    Ok(())
}
