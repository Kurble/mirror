use super::*;
use serde_json::from_value;

pub trait Primitive<'de>: Deserialize<'de> { }

impl<'de, T: for<'e> Primitive<'e>> Reflect<'de> for T {
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

macro_rules! primitive {
    ($($pp:ty,)*) => { $(primitive!($pp);)* };
    ($p:ty) => { impl<'de> Primitive<'de> for $p { } };
}

macro_rules! tuple {
    ($($p:ident),*) => {
        impl<'de, $( $p : Primitive<'de> ),* > Primitive<'de> for ($($p),*) { }
    };
}

primitive!(bool, i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, String, );
tuple!(A, B);
tuple!(A, B, C);
tuple!(A, B, C, D);
tuple!(A, B, C, D, E);
tuple!(A, B, C, D, E, F);
tuple!(A, B, C, D, E, F, G);
tuple!(A, B, C, D, E, F, G, H);
tuple!(A, B, C, D, E, F, G, H, I);
tuple!(A, B, C, D, E, F, G, H, I, J);
tuple!(A, B, C, D, E, F, G, H, I, J, K);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);