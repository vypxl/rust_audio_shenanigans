use std::{error::Error, path::PathBuf};

use egui_file::FileDialog;

mod player;
use player::Player;

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
            let player = Player::from_file(fname).unwrap();
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
