use super::*;
use std::rc::Rc;
use std::cell::RefCell;

/// Context that will manage a list of reply messages.
/// Every command executed through the context will be added to the list of reply messages.
/// After the context is done, the messages will be sent to the remote(s).
#[derive(Clone)]
pub struct Reply {
    reply: Rc<RefCell<Vec<(String, bool)>>>,
    path: String,
}

impl Reply {
    /// Create a new reply context. Takes a `Vec` of `(String, bool)` to put the reply messages in.
    /// The bool in the tuple specifies whether to send the message back to the origin.
    pub fn new(reply: Vec<(String, bool)>) -> Self {
        Self {
            reply: Rc::new(RefCell::new(reply)),
            path: "".to_string(),
        }
    }

    pub fn into_inner(self) -> Vec<(String, bool)> {
        Rc::try_unwrap(self.reply).unwrap().into_inner()
    }
}

impl Context for Reply {
    type Inner = Self;

    fn command<R, S>(&mut self, value: &mut R, cmd: S) -> Result<(), Error> where
        R: Reflect,
        S: AsRef<str>
    {
        value.command_str((), cmd.as_ref())?;
        Ok(self.reply.borrow_mut().push((format!("{}{}", self.path, cmd.as_ref()), true)))
    }

    fn local_command<R, S>(&mut self, value: &mut R, cmd: S) -> Result<(), Error> where
        R: Reflect,
        S: AsRef<str>
    {
        value.command_str((), cmd.as_ref())?;
        Ok(self.reply.borrow_mut().push((format!("{}{}", self.path, cmd.as_ref()), false)))
    }

    fn with_inner<F: FnMut(Self::Inner)>(&mut self, path: &str, mut f: F) {
        f(Reply {
            reply: self.reply.clone(),
            path: format!("{}{}/", self.path, path)
        });
    }
}
