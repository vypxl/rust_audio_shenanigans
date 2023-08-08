use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    partial_wave::PartialWave,
    wave::{Wave, WaveGenerator},
    waves::{constant, ADSREvent, ADSRTrigger, Constant, ADSR},
};

fn midi_note_number_to_frequency<T: Into<f64>>(note: T) -> f64 {
    2.0f64.powf((note.into() - 69.0) / 12.0) * 440.0
}

type InstrumentWave<T: Wave> = impl Wave;
type Keymap<T> = Arc<Mutex<HashMap<usize, (InstrumentWave<T>, ADSRTrigger)>>>;

pub struct PolyInstrument<T: PartialWave> {
    source: T,
    keymap: Keymap<T::Target<Constant>>,
}

#[derive(Clone)]
pub struct PolyInstrumentWave<T: PartialWave> {
    keymap: Keymap<T::Target<Constant>>,
}

impl<W: Wave, T: PartialWave<Target<Constant> = W> + Clone> PolyInstrument<T> {
    fn make_instrument(&self, note: usize) -> (InstrumentWave<W>, ADSRTrigger) {
        let (adsr, trigger) = ADSR::new(0.02, 0.3, 0.5, 0.05);
        let freq = constant(midi_note_number_to_frequency(note as u8));
        let wave = freq >> self.source.clone();
        let wave = wave * adsr;
        (wave, trigger)
    }

    pub fn play(&mut self, key: usize, e: ADSREvent) {
        let mut keymap = self.keymap.lock().unwrap();
        if let Some((_, trigger)) = keymap.get(&key) {
            trigger.trigger(e);
        } else {
            // let (wave, trigger) = self.make_instrument(key);
            let inst = self.make_instrument(key);
            inst.1.trigger(e);
            keymap.insert(key, inst);
        }
        if e == ADSREvent::Release {
            // keymap.remove(&key);
            // TODO: handle release event (remove from keymap after release time somehow)
        }
    }

    pub fn new(source: T) -> (Self, WaveGenerator<PolyInstrumentWave<T>>) {
        let keymap = Arc::new(Mutex::new(HashMap::new()));
        (
            Self {
                source,
                keymap: keymap.clone(),
            },
            PolyInstrumentWave { keymap }.into(),
        )
    }
}

impl<T: PartialWave + Clone> Wave for PolyInstrument<T> {
    fn next_sample(&mut self) -> f64 {
        self.keymap
            .lock()
            .unwrap()
            .values_mut()
            .fold(0.0, |acc, (wave, _)| acc + wave.next_sample())
    }
}

impl<T: PartialWave + Clone> Wave for PolyInstrumentWave<T> {
    fn next_sample(&mut self) -> f64 {
        self.keymap
            .lock()
            .unwrap()
            .values_mut()
            .fold(0.0, |acc, (wave, _)| acc + wave.next_sample())
    }
}
