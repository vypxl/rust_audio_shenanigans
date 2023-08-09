use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use crate::wave::{Wave, WaveGenerator};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ADSREvent {
    /// Velocity is a value between 0 and 127
    Press(u8),
    Release,
}

const NEW_MSG: u32 = 0x80;
const VEL_MASK: u32 = 0x7F;

pub struct Trigger {
    _trigger: Arc<AtomicU32>,
}

impl Trigger {
    pub fn new(trigger: Arc<AtomicU32>) -> Self {
        Self { _trigger: trigger }
    }

    pub fn trigger(&self, e: ADSREvent) {
        let msg = NEW_MSG
            | match e {
                ADSREvent::Press(vel) => vel as u32,
                ADSREvent::Release => 0,
            };
        self._trigger.store(msg, Ordering::Relaxed);
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
    trigger: Arc<AtomicU32>,
}

impl ADSR {
    pub fn new(
        attack: f64,
        decay: f64,
        sustain: f64,
        release: f64,
    ) -> (WaveGenerator<Self>, Trigger) {
        let trigger = Arc::new(AtomicU32::new(0));
        (
            Self {
                attack,
                decay,
                sustain,
                release,
                phase: 1.0 + attack + decay + release,
                level: 0.0,
                hold: false,
                trigger: trigger.clone(),
            }
            .into(),
            Trigger::new(trigger),
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

    fn handle_msg(&mut self) {
        let msg = self.trigger.swap(0, Ordering::Relaxed);
        if msg == 0 {
            return;
        }

        let vel = msg & VEL_MASK;

        match vel {
            0 => {
                self.hold = false;
            }
            _ => {
                self.phase = if self.phase >= self.attack + self.decay
                    && self.phase < self.attack + self.decay + self.release
                {
                    self.get_release_lvl() * self.attack
                } else {
                    0.0
                };
                self.level = vel as f64 / 127.0;
                self.hold = true;
            }
        }
    }
}

impl Wave for ADSR {
    fn next_sample(&mut self) -> f64 {
        self.handle_msg();

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
