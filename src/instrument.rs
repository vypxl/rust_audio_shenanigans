use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    partial_wave::PartialWaveBuilder,
    wave::{Wave, WaveGenerator},
    waves::{constant, ADSREvent, ADSRTrigger, ADSR},
};

fn midi_note_number_to_frequency<T: Into<f64>>(note: T) -> f64 {
    2.0f64.powf((note.into() - 69.0) / 12.0) * 440.0
}

type InstrumentWave = WaveGenerator;
type Keymap = Arc<Mutex<HashMap<usize, (InstrumentWave, ADSRTrigger)>>>;

pub struct PolyInstrument {
    source: Box<dyn Fn() -> PartialWaveBuilder + Send>,
    keymap: Keymap,
}

#[derive(Clone)]
pub struct PolyInstrumentWave {
    keymap: Keymap,
}

impl PolyInstrument {
    fn make_instrument(&self, note: usize) -> (InstrumentWave, ADSRTrigger) {
        let (adsr, trigger) = ADSR::make(0.02, 0.3, 0.5, 0.05);
        let freq = constant(midi_note_number_to_frequency(note as u8));
        let wave = freq >> (self.source)();
        let wave = wave * adsr;
        (wave, trigger)
    }

    pub fn play(&mut self, key: usize, e: ADSREvent) {
        let mut keymap = self.keymap.lock().unwrap();
        if let Some((_, trigger)) = keymap.get(&key) {
            trigger.trigger(e);
        } else {
            let (wave, trigger) = self.make_instrument(key);
            trigger.trigger(e);
            keymap.insert(key, (wave, trigger));
        }
        if e == ADSREvent::Release {
            // keymap.remove(&key);
            // TODO: handle release event (remove from keymap after release time somehow)
        }
    }

    pub fn new(source: Box<dyn Fn() -> PartialWaveBuilder + Send>) -> (Self, WaveGenerator) {
        let keymap = Arc::new(Mutex::new(HashMap::new()));
        (
            Self {
                source,
                keymap: keymap.clone(),
            },
            WaveGenerator::new(PolyInstrumentWave { keymap }),
        )
    }
}

impl Wave for PolyInstrument {
    fn next_sample(&mut self) -> f64 {
        self.keymap
            .lock()
            .unwrap()
            .values_mut()
            .fold(0.0, |acc, (wave, _)| acc + wave.next_sample())
    }
}

impl Wave for PolyInstrumentWave {
    fn next_sample(&mut self) -> f64 {
        self.keymap
            .lock()
            .unwrap()
            .values_mut()
            .fold(0.0, |acc, (wave, _)| acc + wave.next_sample())
    }
}
