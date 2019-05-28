use super::*;
use serde_json::from_value;

impl<T: Reflect> Reflect for Option<T> {
    fn command<C: Context>(&mut self, mut context: C, command: &Command) -> Result<(), Error> {
        match command {
            &Command::Path { ref element, ref command } => {
                if element == "val" {
                    let elem: &mut T = self.as_mut().ok_or(Error::PathError)?;
                    let mut result = Ok(());
                    context.with_inner(element.as_str(), |c| result=elem.command(c, command));
                    result
                } else {
                    Err(Error::PathError)
                }
            },
            &Command::Set { ref value } => {
                *self = Some(from_value(value.clone())?);
                Ok(())
            },
            &Command::Remove { .. } => {
            	*self = None;
                Ok(())
            }
            &_ => Err(Error::IncompatibleCommand),
        }
    }
}