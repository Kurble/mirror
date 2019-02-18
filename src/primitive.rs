use super::*;
use serde_json::from_value;

macro_rules! primitive {
    ($($pp:ty,)*) => {
        $(primitive!($pp);)*
    };
    ($p:ty) => {
        impl<'de> Reflect<'de> for $p {
            fn command(&mut self, command: &Command) -> Result<(), Error> {
                match command {
                    &Command::Set{ ref value } => {
                        *self = from_value(value.clone())?;
                        Ok(())
                    },
                    &_ => Err(Error::IncompatibleCommand),
                }
            }
        }
    };
}

primitive!(bool, i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, String, );