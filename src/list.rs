use super::*;
use serde_json::from_value;
use std::collections::VecDeque;

impl<T: Reflect> Reflect for Vec<T> {
    fn command<C: Context>(&mut self, context: C, command: &Command) -> Result<(), Error> {
        match command {
            &Command::Path { ref element, ref command } => {
                let index: usize = element.parse()?;
                let elem: &mut T = self.get_mut(index).ok_or(Error::PathError)?;

                let mut result = Ok(());
                context.with_inner(element.as_str(), |c| result=elem.command(c, command));
                result
            },
            &Command::Set { ref value } => {
                *self = from_value(value.clone())?;
                Ok(())
            },
            &Command::Push { ref value } => {
                self.push(from_value(value.clone())?);
                Ok(())
            },
            &Command::Pop => {
                self.pop();
                Ok(())
            },
            &Command::Remove { ref key } => {
                self.remove(from_value(key.clone())?);
                Ok(())
            }
            &_ => Err(Error::IncompatibleCommand),
        }
    }
}

impl<T: Reflect> Reflect for VecDeque<T> {
    fn command<C: Context>(&mut self, context: C, command: &Command) -> Result<(), Error> {
        match command {
            &Command::Path { ref element, ref command } => {
                let index: usize = element.parse()?;
                let elem: &mut T = self.get_mut(index).ok_or(Error::PathError)?;

                let mut result = Ok(());
                context.with_inner(element.as_str(), |c| result=elem.command(c, command));
                result
            },
            &Command::Set { ref value } => {
                *self = from_value(value.clone())?;
                Ok(())
            },
            &Command::Push { ref value } => {
                self.push_back(from_value(value.clone())?);
                Ok(())
            },
            &Command::Pop => {
                self.pop_back();
                Ok(())
            },
            &Command::Remove { ref key } => {
                self.remove(from_value(key.clone())?);
                Ok(())
            }
            &_ => Err(Error::IncompatibleCommand),
        }
    }
}

macro_rules! array {
    ($($nn:expr,)*) => { $(array!($nn);)* };
    ($n:expr) => {
        impl<T: Reflect> Reflect for [T; $n] {
            fn command<C: Context>(&mut self, context: C, command: &Command) -> Result<(), Error> {
                match command {
                    &Command::Path { ref element, ref command } => {
                        let index: usize = element.parse()?;
                        let elem: &mut T = self.get_mut(index).ok_or(Error::PathError)?;
                        let mut result = Ok(());
                        context.with_inner(element.as_str(), |c| {
                            result = elem.command(c, command).map(|_|());
                        });
                        result
                    },
                    &Command::Set { ref value } => {
                        *self = from_value(value.clone())?;
                        Ok(())
                    },
                    &_ => Err(Error::IncompatibleCommand),
                }
            }
        }
    };
}

array!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
       26, 27, 28, 29, 30, 31, 32, );