use tokio::sync::watch::{channel, Receiver, Sender};

use crate::wave::{Wave, WaveGenerator};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ADSREvent {
    Press(f64),
    Release,
}

pub struct Trigger {
    tx: Sender<ADSREvent>,
}

impl Trigger {
    pub fn new(tx: Sender<ADSREvent>) -> Self {
        Self { tx }
    }

    pub fn trigger(&self, e: ADSREvent) {
        // Ignoring this error, because it's not important. The send only fails, if the
        // receiver is dropped, and that only happens, if the audio thread dies. This is easily
        // detectable.
        let _ = self.tx.send(e);
    }
}

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
    pub fn make(attack: f64, decay: f64, sustain: f64, release: f64) -> (WaveGenerator, Trigger) {
        let (tx, rx) = channel(ADSREvent::Release);
        (
            WaveGenerator::new(Self {
                attack,
                decay,
                sustain,
                release,
                phase: 1.0 + attack + decay + release,
                level: 0.0,
                hold: false,
                event_queue: rx,
            }),
            Trigger::new(tx),
        )
    }

    fn get_attack_lvl(&self) -> f64 {
        self.phase / self.attack
    }

    fn get_decay_lvl(&self) -> f64 {
        1.0 - (self.phase - self.attack) / self.decay * (1.0 - self.sustain)
    }

    fn get_sustain_lvl(&self) -> f64 {
        self.sustain
    }

    fn get_release_lvl(&self) -> f64 {
        self.sustain * (1.0 - (self.phase - (self.attack + self.decay)) / self.release)
    }
}

impl Wave for ADSR {
    fn next_sample(&mut self) -> f64 {
        if let Ok(has_changed) = self.event_queue.has_changed() {
            if has_changed {
                let m = *self.event_queue.borrow_and_update();
                match m {
                    ADSREvent::Press(vel) => {
                        self.phase = if self.phase >= self.attack + self.decay
                            && self.phase < self.attack + self.decay + self.release
                        {
                            self.get_release_lvl() * self.attack
                        } else {
                            0.0
                        };
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

        let v = if self.phase < self.attack {
            self.get_attack_lvl()
        } else if self.phase < self.attack + self.decay {
            self.get_decay_lvl()
        } else if self.hold {
            self.get_sustain_lvl()
        } else {
            self.get_release_lvl()
        };

        v * self.level
    }
}
