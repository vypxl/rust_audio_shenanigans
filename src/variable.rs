use std::{
    ops::{Deref, DerefMut},
    sync::mpsc::{Receiver, SendError, Sender},
};

pub struct Variable<T> {
    pub value: T,
    rx: Receiver<T>,
}

pub struct VariableUpdater<T> {
    tx: Sender<T>,
}

impl<T> Variable<T> {
    pub fn new(value: T) -> (Self, VariableUpdater<T>) {
        let (tx, rx) = std::sync::mpsc::channel();
        (Self { value, rx }, VariableUpdater { tx })
    }

    pub fn update(&mut self) -> &T {
        while let Ok(value) = self.rx.try_recv() {
            self.value = value;
        }

        &self.value
    }

    pub fn update_once(&mut self) -> &T {
        if let Ok(value) = self.rx.try_recv() {
            self.value = value;
        }

        &self.value
    }

    pub fn update_mut(&mut self) -> &mut T {
        self.update();
        &mut self.value
    }

    pub fn update_once_mut(&mut self) -> &mut T {
        self.update_once();
        &mut self.value
    }
}

impl<T> From<T> for Variable<T> {
    fn from(value: T) -> Self {
        Self::new(value).0
    }
}

impl<T> VariableUpdater<T> {
    pub fn update(&self, value: T) -> Result<(), SendError<T>> {
        self.tx.send(value)
    }
}

impl<T> Deref for Variable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Variable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
