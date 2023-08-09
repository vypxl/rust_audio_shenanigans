use criterion::{criterion_group, criterion_main, Criterion};
use rust_audio_shenanigans::{effects::lowpass, instrument::*, wave::Wave, waves::*};

pub fn sine_lowpass(c: &mut Criterion) {
    let mut g = c.benchmark_group("sine_lowpass");
    g.bench_function("sine_lowpass", |b| {
        let mut wave = ((constant(50) >> sine()) * 0.1) >> lowpass(5000.0, 1.1);
        b.iter(|| wave.next_sample());
    });
    g.finish();
}

pub fn mountain_king(c: &mut Criterion) {
    let mut g = c.benchmark_group("mountain_king");
    g.bench_function("mountain_king", |b| {
        let song = midly::Smf::parse(include_bytes!("../songs/grieg_mountain_king.mid")).unwrap();
        let mut song_player = get_song_player(44100, song);
        b.iter(&mut song_player);
    });
    g.finish();
}

criterion_group!(benches, mountain_king, sine_lowpass);
criterion_main!(benches);

fn process_event(
    event: midly::TrackEvent,
    trigger: &mut impl FnMut(usize, ADSREvent),
    mspt: &mut f32,
    tpb: u32,
) {
    match event.kind {
        midly::TrackEventKind::Midi { message, .. } => match message {
            midly::MidiMessage::NoteOn { key, vel } => {
                if vel.as_int() == 0 {
                    trigger(key.as_int() as usize, ADSREvent::Release);
                    return;
                }
                trigger(key.as_int() as usize, ADSREvent::Press(vel.as_int()));
            }
            midly::MidiMessage::NoteOff { key, .. } => {
                // todo
                trigger(key.as_int() as usize, ADSREvent::Release);
            }
            _ => {}
        },
        midly::TrackEventKind::Meta(meta) => match meta {
            midly::MetaMessage::Tempo(tempo) => {
                *mspt = tempo.as_int() as f32 / (1.0 * tpb as f32);
            }
            midly::MetaMessage::TrackName(_) => {}
            _ => {}
        },
        _ => {}
    }
}

fn get_song_player(sample_rate: u32, song: midly::Smf) -> impl FnMut() {
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

    let (mut inst, wave) = PolyInstrument::new(triangle() >> lowpass(600.0, 1.0));
    let mut wave = wave * 0.2;
    // let (mut inst, wave) = PolyInstrument::new(sawtooth());
    // let (mut inst, wave) = PolyInstrument::new(sine());

    // Process all events in order.
    let mut last_timestamp = 0;
    let mut mspt = 0.0;
    let mut i = 0;
    let mut sleepsamples = 0;

    move || {
        // for (timestamp, event) in &all_events {
        //     let delta = (timestamp - last_timestamp) as f32;
        //     let sleeptime = ((delta * mspt) / 1000.0) as u64;
        //     let mut sleepsamples = ((sleeptime as f32) * (sample_rate as f32) / 1000.0) as usize;
        //     // println!("sleeping for {sleeptime}ms or {sleepsamples} samples");
        //     while sleepsamples > 0 {
        //         wave.next_sample();
        //         sleepsamples -= 1;
        //     }
        //     process_event(*event, &mut |key, e| inst.play(key, e), &mut mspt, tpb);
        //     last_timestamp = *timestamp;
        // }
        if sleepsamples > 0 {
            wave.next_sample();
            sleepsamples -= 1;
            return;
        }

        let (timestamp, event) = &all_events[i];
        let delta = (timestamp - last_timestamp) as f32;
        let sleeptime = ((delta * mspt) / 1000.0) as u64;
        sleepsamples = ((sleeptime as f32) * (sample_rate as f32) / 1000.0) as usize;
        process_event(*event, &mut |key, e| inst.play(key, e), &mut mspt, tpb);
        last_timestamp = *timestamp;

        i = (i + 1) % all_events.len();
    }
}
