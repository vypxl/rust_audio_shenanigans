use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    partial_wave::PartialWave,
    wave::{Wave, WaveGenerator},
    waves::{constant, misc::MixWaveSource, ADSREvent, ADSRTrigger, Constant, ADSR},
};

fn midi_note_number_to_frequency<T: Into<f64>>(note: T) -> f64 {
    2.0f64.powf((note.into() - 69.0) / 12.0) * 440.0
}

type InstrumentWave<T> = WaveGenerator<MixWaveSource<<T as PartialWave<Constant>>::Target, ADSR>>;
type Keymap<T> = Arc<Mutex<HashMap<usize, (InstrumentWave<T>, ADSRTrigger)>>>;

pub struct PolyInstrument<T: PartialWave<Constant> + Clone> {
    source: T,
    keymap: Keymap<T>,
}

#[derive(Clone)]
pub struct PolyInstrumentWave<T: PartialWave<Constant> + Clone> {
    keymap: Keymap<T>,
}

impl<U: Wave + Clone, T: PartialWave<Constant, Target = U> + Clone> PolyInstrument<T> {
    fn make_instrument(&self, note: usize) -> (InstrumentWave<T>, ADSRTrigger) {
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
            let (wave, trigger) = self.make_instrument(key);
            trigger.trigger(e);
            keymap.insert(key, (wave, trigger));
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

impl<U: Wave + Clone, T: PartialWave<Constant, Target = U> + Clone> Wave for PolyInstrument<T> {
    fn next_sample(&mut self) -> f64 {
        self.keymap
            .lock()
            .unwrap()
            .values_mut()
            .fold(0.0, |acc, (wave, _)| acc + wave.next_sample())
    }
}

impl<U: Wave + Clone, T: PartialWave<Constant, Target = U> + Clone> Wave for PolyInstrumentWave<T> {
    fn next_sample(&mut self) -> f64 {
        self.keymap
            .lock()
            .unwrap()
            .values_mut()
            .fold(0.0, |acc, (wave, _)| acc + wave.next_sample())
    }
}
