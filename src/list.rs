use super::*;
use serde_json::from_value;

impl<'de, T: for<'e> Reflect<'e>> Reflect<'de> for Vec<T> {
    fn command(&mut self, command: &Command) -> Result<(), Error> {
        match command {
            &Command::Path { ref element, ref command } => {
                let index: usize = element.parse()?;
                let elem: &mut T = self.get_mut(index).ok_or(Error::PathError)?;
                Ok(elem.command(command)?)
            },
            &Command::Set { ref value } => {
                *self = from_value(value.clone())?;
                Ok(())
            }
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