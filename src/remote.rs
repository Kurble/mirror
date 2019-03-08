use crate::error::Error;

/// Trait for communication with remote nodes
pub trait Remote {
    /// Close the connection to the remote node.
    /// If the underlying protocol supports a graceful disconnect this will perform
    ///  a graceful disconnect.
    fn close(&mut self);

    /// Returns whether the connection to the remote node is still alive.
    fn alive(&self) -> bool;

    /// Sends a message to the remote node. If sending the message fails, `Err` will be returned.
    /// Otherwise, the result will be `Ok`
    fn send(&mut self, message: &str) -> Result<(), Error>;

    /// Try to receive a message from the remote node. If no message is available at this time,
    /// `None` will be returned. If the connection to the remote node is closed this function will
    /// also return `None`. Only if a message is available it will be returned.
    /// This function never blocks.
    fn recv(&mut self) -> Option<String>;

    /// Returns an iterator over the available messages. When there are no more messages available
    /// at this time the iterator will yield `None`. The returned iterator will never block.
    fn iter(&mut self) -> Iter<Self> {
        Iter(self)
    }
}

/// Iterates over the available messages from a remote node.
pub struct Iter<'a, R: 'a + Remote + ?Sized>(&'a mut R);

impl<'a, R: Remote> Iterator for Iter<'a, R> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.0.recv()
    }
}
