use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
};
use tokio::sync::watch::{channel, Receiver};

#[derive(Clone)]
pub enum Variable<T> {
    Static {
        value: T,
        rx: Receiver<T>,
    },
    Dynamic {
        lock_value: Arc<RwLock<T>>,
        value: T,
    },
}

pub trait VariableSetter<T>: Fn(T) {}
impl<T: Fn(U), U> VariableSetter<U> for T {}

pub type VariableHandle<T> = Arc<RwLock<T>>;

impl<T> Variable<T>
where
    T: 'static + Clone,
{
    pub fn new_dynamic(value: T) -> (Self, VariableHandle<T>) {
        let lock_value = Arc::new(RwLock::new(value.clone()));
        (
            Self::Dynamic {
                value: value.clone(),
                lock_value: Arc::new(RwLock::new(value)).clone(),
            },
            lock_value,
        )
    }
}

impl<T> Variable<T>
where
    T: Clone,
{
    pub fn new(value: T) -> (Self, impl VariableSetter<T>) {
        let (tx, rx) = channel(value.clone());
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
                if let Ok(has_changed) = rx.has_changed() {
                    if has_changed {
                        *value = rx.borrow_and_update().clone();
                    }
                }
                value
            }
            Self::Dynamic { value, lock_value } => {
                if let Ok(lock_value) = lock_value.read() {
                    *value = lock_value.clone();
                }
                value
            }
        }
    }

    pub fn update_mut(&mut self) -> &mut T {
        self.update();
        self.value_mut()
    }
}

impl<T> From<T> for Variable<T>
where
    T: Clone,
{
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
