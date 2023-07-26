use std::{
    ops::{Deref, DerefMut},
    sync::mpsc::Receiver,
};

type UpdateFun<T> = Box<dyn FnOnce(&mut T) + Send>;

pub enum Variable<T> {
    Static {
        value: T,
        rx: Receiver<T>,
    },
    Dynamic {
        value: T,
        rx: Receiver<UpdateFun<T>>,
    },
}

pub trait VariableSetter<T>: Fn(T) {}
impl<T: Fn(U), U> VariableSetter<U> for T {}

pub trait VariableUpdater<T>: Fn(UpdateFun<T>) {}
impl<T: Fn(UpdateFun<U>) + 'static, U> VariableUpdater<U> for T {}

impl<T: 'static> Variable<T> {
    pub fn new_dynamic(value: T) -> (Self, impl VariableUpdater<T>) {
        let (tx, rx) = std::sync::mpsc::channel();
        (Self::Dynamic { value, rx }, move |t: UpdateFun<T>| {
            // Ignoring this error, because it's not important. The send only fails, if the
            // receiver is dropped, and that only happens, if the audio thread dies. This is easily
            // detectable.
            let _ = tx.send(t);
        })
    }
}

impl<T> Variable<T> {
    pub fn new(value: T) -> (Self, impl VariableSetter<T>) {
        let (tx, rx) = std::sync::mpsc::channel();
        (Self::Static { value, rx }, move |t: T| {
            // Ignoring this error, because it's not important. The send only fails, if the
            // receiver is dropped, and that only happens, if the audio thread dies. This is easily
            // detectable.
            let _ = tx.send(t);
        })
    }

    pub fn value(&self) -> &T {
        match self {
            Self::Static { value, .. } => value,
            Self::Dynamic { value, .. } => value,
        }
    }

    pub fn value_mut(&mut self) -> &mut T {
        match self {
            Self::Static { value, .. } => value,
            Self::Dynamic { value, .. } => value,
        }
    }

    pub fn update(&mut self) -> &T {
        match self {
            Self::Static { value, rx } => {
                while let Ok(new_value) = rx.try_recv() {
                    *value = new_value;
                }
                value
            }
            Self::Dynamic { value, rx } => {
                while let Ok(updater) = rx.try_recv() {
                    updater(value);
                }
                value
            }
        }
    }

    pub fn update_once(&mut self) -> &T {
        match self {
            Self::Static { value, rx } => {
                if let Ok(new_value) = rx.try_recv() {
                    *value = new_value;
                }
                value
            }
            Self::Dynamic { value, rx } => {
                if let Ok(updater) = rx.try_recv() {
                    updater(value);
                }
                value
            }
        }
    }

    pub fn update_mut(&mut self) -> &mut T {
        self.update();
        self.value_mut()
    }

    pub fn update_once_mut(&mut self) -> &mut T {
        self.update_once();
        self.value_mut()
    }
}

impl<T> From<T> for Variable<T> {
    fn from(value: T) -> Self {
        Self::new(value).0
    }
}

impl<T> Deref for Variable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Static { value, .. } => value,
            Self::Dynamic { value, .. } => value,
        }
    }
}

impl<T> DerefMut for Variable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Static { value, .. } => value,
            Self::Dynamic { value, .. } => value,
        }
    }
}
