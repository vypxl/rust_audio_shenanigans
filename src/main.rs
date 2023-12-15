use std::{
    error::Error,
    path::PathBuf,
    thread::{self, JoinHandle},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use egui_file::FileDialog;
use rust_audio_shenanigans::{
    effects::lowpass,
    instrument::*,
    partial_wave::{PartialWave, PartialWaveBuilder},
    waves::*,
    *,
};

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

fn instrument() -> PartialWaveBuilder<impl PartialWave + Clone> {
    let wave = triangle();
    let wave2 = ((pass() * 2) >> square()) * 0.5;
    let wave3 = ((pass() * 3) >> square()) * 0.25;
    let wave4 = ((pass() * 4) >> square()) * 0.125;
    let wave5 = ((pass() * 5) >> square()) * 0.0625;
    let wave6 = ((pass() * 6) >> square()) * 0.03125;
    let wave7 = ((pass() * 7) >> square()) * 0.015625;
    let wave8 = ((pass() * 8) >> square()) * 0.0078125;

    ((wave + wave2 + wave3 + wave4 + wave5 + wave6 + wave7 + wave8) * 0.2) >> lowpass(5000.0, 1.0)
}

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
                    // println!("off");
                    trigger(key.as_int() as usize, ADSREvent::Release);
                    return;
                }
                // println!("triggering: {key} {vel}");
                trigger(key.as_int() as usize, ADSREvent::Press(vel.as_int()));
            }
            midly::MidiMessage::NoteOff { key, .. } => {
                // todo
                trigger(key.as_int() as usize, ADSREvent::Release);
                // println!("off");
            }
            _x => {
                // println!("midi: {:?}", _x);
            }
        },
        midly::TrackEventKind::Meta(meta) => match meta {
            midly::MetaMessage::Tempo(tempo) => {
                *mspt = tempo.as_int() as f32 / (1.0 * tpb as f32);
                // println!("tempo: {}, tpb: {}, mspt: {}", tempo, tpb, mspt);
            }
            midly::MetaMessage::TrackName(_name) => {
                // println!("track name: {}", String::from_utf8_lossy(_name));
            }
            _x => {
                // println!("meta: {:?}", _x);
            }
        },
        _x => {
            // println!("other: {:?}", _x);
        }
    }
}

fn setup_streamer(sample_rate: u32, song: midly::Smf) -> (WaveStreamer, JoinHandle<()>) {
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

    let p = instrument();
    let (mut inst, wave) = PolyInstrument::new(p);
    let wave = wave * 0.1;

    // Process all events in order.
    let handle = thread::spawn(move || {
        let mut last_timestamp = 0;
        let mut mspt = 0.0;
        for (timestamp, event) in all_events {
            let delta = (timestamp - last_timestamp) as f32;
            let sleeptime = (delta * mspt) as u64;
            if sleeptime > 0 {
                thread::sleep(std::time::Duration::from_micros((delta * mspt) as u64))
            }
            process_event(event, &mut |key, e| inst.play(key, e), &mut mspt, tpb);
            last_timestamp = timestamp;
        }
    });

    (WaveStreamer::new(wave, sample_rate), handle)
}

struct Player {
    stream: cpal::Stream,
    _thread_handle: Option<JoinHandle<()>>,
}

impl Player {
    fn new(stream: cpal::Stream, thread_handle: JoinHandle<()>) -> Self {
        Self {
            stream,
            _thread_handle: Some(thread_handle),
        }
    }
    fn play(&self) -> Result<(), Box<dyn Error>> {
        self.stream.play()?;
        Ok(())
    }
    fn stop(&self) -> Result<(), Box<dyn Error>> {
        self.stream.pause()?;
        Ok(())
    }

    // fn wait(&mut self) {
    //     if let Some(h) = self._thread_handle.take() {
    //         h.join().unwrap();
    //     }
    // }
}

fn play_song_from_file(fname: &str) -> Result<Player, Box<dyn Error>> {
    let (device, config) = setup_device()?;

    let data = std::fs::read(fname)?;
    let smf = midly::Smf::parse(&data)?;
    let (streamer, handle) = setup_streamer(config.sample_rate.0, smf);

    let stream = setup_stream(&device, &config, streamer)?;

    Ok(Player::new(stream, handle))
}

fn main() -> Result<(), Box<dyn Error>> {
    let fname = std::env::args().nth(1).unwrap_or_default();

    open_gui(fname)?;
    Ok(())
}

fn open_gui(fname: String) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "vypxl's Rust Audio Shenanigans",
        options,
        Box::new(|_| Box::new(App::new(fname))),
    )
}

#[derive(Default)]
struct App {
    fname: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
    player: Option<Player>,
}

impl App {
    fn new(fname: String) -> Self {
        let should_play = !fname.is_empty();
        let mut ret = Self {
            fname: Some(PathBuf::from(fname)),
            ..Default::default()
        };

        if should_play {
            ret.play();
        }

        ret
    }

    fn play(&mut self) {
        if let Some(fname) = &self.fname {
            let fname = fname.to_str().unwrap();
            if let Some(h) = self.player.take() {
                let _ = h.stop();
            }
            println!("Playing {}", fname);
            let player = play_song_from_file(fname).unwrap();
            let _ = player.play();
            self.player = Some(player);
        } else {
            println!("No file selected!");
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("vypxl's Rust Audio Shenanigans");
                ui.label("Choose a MIDI file to play:");
                ui.horizontal(|ui| {
                    let lab = ui.label("Path: ");
                    let mut p: String = self
                        .fname
                        .clone()
                        .unwrap_or_default()
                        .into_os_string()
                        .into_string()
                        .unwrap_or_default();
                    let edit = ui.text_edit_singleline(&mut p).labelled_by(lab.id);
                    if edit.changed() {
                        self.fname = Some(PathBuf::from(p));
                    }

                    if (ui.button("Choose File")).clicked() {
                        let mut dialog = FileDialog::open_file(self.fname.clone());
                        dialog.open();
                        self.open_file_dialog = Some(dialog);
                    }
                });

                if let Some(dialog) = &mut self.open_file_dialog {
                    if dialog.show(ctx).selected() {
                        if let Some(file) = dialog.path() {
                            self.fname = Some(file.to_path_buf());
                        }
                    }
                }

                ui.horizontal(|ui| {
                    if ui.button("Play").clicked() {
                        self.play()
                    }
                    if ui.button("Stop").clicked() {
                        if let Some(h) = self.player.take() {
                            let _ = h.stop();
                        }
                    }
                });
            });
        });
    }
}
