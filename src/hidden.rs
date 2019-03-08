use super::*;
use std::ops::{Deref, DerefMut};

pub struct Hidden<T>(Option<T>);

impl<T> Hidden<T> {
    pub fn new(value: T) -> Self {
        Hidden(Some(value))
    }

    pub fn as_ref(&self) -> Option<&T> {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        self.0.as_mut()
    }
}

impl<T> Deref for Hidden<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.as_ref().unwrap()
    }
}

impl<T> DerefMut for Hidden<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut().unwrap()
    }
}

impl<T> Serialize for Hidden<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

impl<'de, T> Deserialize<'de> for Hidden<T> {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Ok(Hidden(None))
    }
}

impl<'de, T> Reflect<'de> for Hidden<T> {
    fn command<C: Context>(&mut self, _: C, _: &Command) -> Result<(), Error> {
        Err(Error::IncompatibleCommand)
    }
}
