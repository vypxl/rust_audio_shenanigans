use tokio::sync::watch::{channel, Receiver};

use crate::wave::{Wave, WaveGenerator};

#[derive(Debug, Clone, Copy)]
pub enum ADSREvent {
    Press(f64),
    Release,
}

pub trait Trigger: Fn(ADSREvent) {}
impl<T: Fn(ADSREvent)> Trigger for T {}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
pub struct ADSR {
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
    phase: f64,
    level: f64,
    hold: bool,
    event_queue: Receiver<ADSREvent>,
}

impl ADSR {
    pub fn new(
        attack: f64,
        decay: f64,
        sustain: f64,
        release: f64,
    ) -> (WaveGenerator<Self>, impl Trigger) {
        let (tx, rx) = channel(ADSREvent::Release);
        (
            Self {
                attack,
                decay,
                sustain,
                release,
                phase: 1.0 + attack + decay + release,
                level: 0.0,
                hold: false,
                event_queue: rx,
            }
            .into(),
            move |e| {
                // Ignoring this error, because it's not important. The send only fails, if the
                // receiver is dropped, and that only happens, if the audio thread dies. This is easily
                // detectable.
                let _ = tx.send(e);
            },
        )
    }
}

impl Wave for ADSR {
    fn next_sample(&mut self) -> f64 {
        if let Ok(has_changed) = self.event_queue.has_changed() {
            if has_changed {
                match *self.event_queue.borrow_and_update() {
                    ADSREvent::Press(vel) => {
                        self.phase = 0.0;
                        self.level = vel;
                        self.hold = true;
                    }
                    ADSREvent::Release => {
                        self.hold = false;
                    }
                }
            }
        }

        if self.phase >= self.attack + self.decay + self.release {
            return 0.0;
        }

        if self.phase <= self.attack + self.decay || !self.hold {
            self.phase += 1.0 / 44100.0;
        }

        let phase = self.phase;
        let attack = self.attack;
        let decay = self.decay;
        let sustain = self.sustain;
        let release = self.release;
        let v = if phase < attack {
            phase / attack
        } else if phase < attack + decay {
            1.0 - (phase - attack) / decay * (1.0 - sustain)
        } else if self.hold {
            sustain
        } else {
            sustain * (1.0 - (phase - (attack + decay)) / release)
        };

        v * self.level
    }
}
