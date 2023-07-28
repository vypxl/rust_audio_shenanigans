use tokio::sync::watch::{channel, Receiver};

use crate::wave::{Wave, WaveGenerator};

pub trait Trigger: Fn() {}
impl<T: Fn()> Trigger for T {}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
pub struct ADSR {
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
    phase: f64,
    trigger: Receiver<u8>,
}

impl ADSR {
    pub fn new(
        attack: f64,
        decay: f64,
        sustain: f64,
        release: f64,
    ) -> (WaveGenerator<Self>, impl Trigger) {
        let (tx, rx) = channel(0);
        (
            Self {
                attack,
                decay,
                sustain,
                release,
                phase: 0.0,
                trigger: rx,
            }
            .into(),
            move || {
                // Ignoring this error, because it's not important. The send only fails, if the
                // receiver is dropped, and that only happens, if the audio thread dies. This is easily
                // detectable.
                let _ = tx.send(0);
            },
        )
    }
}

impl Wave for ADSR {
    fn next_sample(&mut self) -> f64 {
        if let Ok(has_changed) = self.trigger.has_changed() {
            if has_changed {
                self.phase = 0.0;
                self.trigger.borrow_and_update();
            }
        }

        if self.phase >= self.attack + self.decay + self.release {
            return 0.0;
        }
        self.phase += 1.0 / 44100.0;

        let phase = self.phase;
        let attack = self.attack;
        let decay = self.decay;
        let sustain = self.sustain;
        let release = self.release;
        if phase < attack {
            phase / attack
        } else if phase < attack + decay {
            1.0 - (phase - attack) / decay * (1.0 - sustain)
        // } else if phase < attack + decay - release {
        // sustain
        } else {
            sustain * (1.0 - (phase - (attack + decay)) / release)
        }
    }
}
