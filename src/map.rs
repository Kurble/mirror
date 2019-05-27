use super::*;
use serde_json::from_value;
use serde_json::from_str;
use std::collections::HashMap;
use std::hash::Hash;

impl<K: Primitive + Eq + Hash, V: Reflect> Reflect for HashMap<K, V> {
    fn command<C: Context>(&mut self, context: C, command: &Command) -> Result<(), Error> {
        match command {
            &Command::Path { ref element, ref command } => {

                let index: K = from_value(from_str(element.as_str())?)?;
                let elem: &mut V = self.get_mut(&index).ok_or(Error::PathError)?;

                let mut result = Ok(());
                context.with_inner(element.as_str(), |c| result=elem.command(c, command));
                result
            },
            &Command::Set { ref value } => {
                *self = from_value(value.clone())?;
                Ok(())
            },
            &Command::Insert { ref key, ref value } => {
                //self.push(from_value(value.clone())?);
                let key: K = from_value(key.clone())?;
                let value: V = from_value(value.clone())?;
                self.insert(key, value);
                Ok(())
            },
            &Command::Remove { ref key } => {
            	let key: K = from_value(key.clone())?;
                self.remove(&key);
                Ok(())
            }
            &_ => Err(Error::IncompatibleCommand),
        }
    }
}