use std::{error::Error, thread};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use rust_audio_shenanigans::{wave::Wave, waves::*, *};

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
        config,
        move |data: &mut [f32], _| streamer.generate(data),
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn midi_note_number_to_frequency(note: midly::num::u7) -> f64 {
    2.0f64.powf((note.as_int() as f64 - 69.0) / 12.0) * 440.0
}

fn instrument() -> (impl Wave + Clone, impl Fn(ADSREvent), impl Fn(f64)) {
    // let wave = (((constant(20) >> sine() >> triangle()) * 100 + 650) >> saw())
    // * ((constant(80) >> sine()) + 0.5);
    let (adsr, trigger) = ADSR::new(0.08, 0.08, 0.8, 0.1);
    let (freq, set_freq) = var(440.0);
    let wave = freq.clone() >> triangle();
    let wave2 = ((freq.clone() * 2) >> triangle()) * 0.5;
    let wave3 = ((freq.clone() * 3) >> triangle()) * 0.25;
    let wave4 = ((freq.clone() * 4) >> triangle()) * 0.125;
    let wave5 = ((freq.clone() * 5) >> triangle()) * 0.0625;
    let wave6 = ((freq.clone() * 6) >> triangle()) * 0.03125;
    let wave7 = ((freq.clone() * 7) >> triangle()) * 0.015625;
    let wave8 = ((freq.clone() * 8) >> triangle()) * 0.0078125;
    let wave = (wave + wave2 + wave3 + wave4 + wave5 + wave6 + wave7 + wave8) * 0.2 * adsr;

    (wave, trigger, set_freq)
}

fn instrument_poly(n: usize) -> (impl Wave + Clone, impl Fn(usize, ADSREvent)) {
    let (wave, trigger, set_freq) = instrument();
    let mut waves = vec![wave];
    let mut triggers = vec![trigger];
    let mut set_freqs = vec![set_freq];

    for _ in 1..n {
        let (wave, trigger, set_freq) = instrument();
        waves.push(wave);
        triggers.push(trigger);
        set_freqs.push(set_freq);
    }

    let trigger = move |key: usize, e: ADSREvent| {
        println!("triggering {}", key);
        set_freqs[key % n](midi_note_number_to_frequency((key as u8).into()));
        triggers[key % n](e);
    };

    (waves, trigger)
}

fn process_event(
    event: midly::TrackEvent,
    trigger: &impl Fn(usize, ADSREvent),
    mspt: &mut f32,
    tpb: u32,
) {
    match event.kind {
        midly::TrackEventKind::Midi { message, .. } => match message {
            midly::MidiMessage::NoteOn { key, vel } => {
                if vel.as_int() == 0 {
                    println!("off");
                    trigger(key.as_int() as usize, ADSREvent::Release);
                    return;
                }
                trigger(
                    key.as_int() as usize,
                    ADSREvent::Press(vel.as_int() as f64 / 127.0),
                );
            }
            midly::MidiMessage::NoteOff { key, .. } => {
                // todo
                trigger(key.as_int() as usize, ADSREvent::Release);
                println!("off");
            }
            x => {
                println!("midi: {:?}", x);
            }
        },
        midly::TrackEventKind::Meta(meta) => match meta {
            midly::MetaMessage::Tempo(tempo) => {
                *mspt = tempo.as_int() as f32 / (1.0 * tpb as f32);
                println!("tempo: {}, tpb: {}, mspt: {}", tempo, tpb, mspt);
            }
            midly::MetaMessage::TrackName(name) => {
                println!("track name: {}", String::from_utf8_lossy(name));
            }
            x => {
                println!("meta: {:?}", x);
            }
        },
        x => {
            println!("other: {:?}", x);
        }
    }
}

fn setup_streamer(sample_rate: u32, song: midly::Smf) -> WaveStreamer {
    let tpb = match song.header.timing {
        midly::Timing::Metrical(x) => x.as_int() as u32,
        midly::Timing::Timecode(_, _) => todo!(),
    };

    let mut all_events = Vec::new();

    for track in song.to_static().tracks.into_iter() {
        let mut cursor = 0;
        for event in track.into_iter() {
            cursor += event.delta.as_int();
            all_events.push((cursor, event));
        }
    }

    // Sort all events by their timestamp.
    all_events.sort_by_key(|(timestamp, _)| *timestamp);

    let (waves, trigger) = instrument_poly(31);

    // Process all events in order.
    thread::spawn(move || {
        let mut last_timestamp = 0;
        let mut mspt = 0.0;
        for (timestamp, event) in all_events {
            let delta = (timestamp - last_timestamp) as f32;
            let sleeptime = (delta * mspt) as u64;
            if sleeptime > 0 {
                thread::sleep(std::time::Duration::from_micros((delta * mspt) as u64))
            }
            print!("{delta} ");
            process_event(event, &trigger, &mut mspt, tpb);
            last_timestamp = timestamp;
        }
    });

    WaveStreamer::new(waves.clone(), waves, sample_rate)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (device, config) = setup_device()?;

    let fname = std::env::args().nth(1).unwrap();
    let data = std::fs::read(fname)?;
    let smf = midly::Smf::parse(&data)?;
    let streamer = setup_streamer(config.sample_rate.0, smf);

    let stream = setup_stream(&device, &config, streamer)?;
    stream.play()?;

    std::thread::park();

    Ok(())
}
