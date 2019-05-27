use super::*;

/// Context that will manage a list of reply messages.
/// Every command executed through the context will be added to the list of reply messages.
/// After the context is done, the messages will be sent to the remote(s).
pub struct Reply<'a> {
    reply: &'a mut Vec<(String, bool)>,
    path: String,
}

impl<'a> Reply<'a> {
    /// Create a new reply context. Takes a `Vec` of `(String, bool)` to put the reply messages in.
    /// The bool in the tuple specifies whether to send the message back to the origin.
    pub fn new(reply: &'a mut Vec<(String, bool)>) -> Self {
        Self {
            reply,
            path: "".to_string(),
        }
    }
}

impl<'a> Context for Reply<'a> {
    type Inner = Self;

    fn command<R, S>(&mut self, value: &mut R, cmd: S) -> Result<(), Error> where
        R: Reflect,
        S: AsRef<str>
    {
        value.command_str((), cmd.as_ref())?;
        Ok(self.reply.push((format!("{}{}", self.path, cmd.as_ref()), true)))
    }

    fn local_command<R, S>(&mut self, value: &mut R, cmd: S) -> Result<(), Error> where
        R: Reflect,
        S: AsRef<str>
    {
        value.command_str((), cmd.as_ref())?;
        Ok(self.reply.push((format!("{}{}", self.path, cmd.as_ref()), false)))
    }

    fn with_inner<F: FnMut(Self::Inner)>(self, path: &str, mut f: F) {
        f(Reply {
            reply: &mut * self.reply,
            path: format!("{}{}/", self.path, path)
        });
    }
}
