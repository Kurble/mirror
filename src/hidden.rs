use super::*;

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
    fn command(&mut self, _: &Command) -> Result<(), Error> {
        Err(Error::IncompatibleCommand)
    }
}
