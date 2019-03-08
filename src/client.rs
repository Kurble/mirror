use super::*;

use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::marker::PhantomData;
use serde_json::{Value, from_value};
use futures::*;

pub struct Client<T: for<'a> Reflect<'a>, R: Remote> {
    value: T,
    remote: R,
}

struct Connect<T: for<'a> Reflect<'a>, R: Remote> {
    remote: Option<R>,
    ph: PhantomData<T>,
}

impl<T: for<'a> Reflect<'a>, R: Remote> Deref for Client<T, R> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: for<'a> Reflect<'a>, R: Remote> DerefMut for Client<T, R> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: for<'a> Reflect<'a>, R: Remote> Future for Connect<T, R> {
    type Item = Client<T, R>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(message) = self.remote.as_mut().unwrap().recv() {
            let value: Value = Value::from_str(message.as_str())?;
            let value: T = from_value(value)?;

            Ok(Async::Ready(Client {
                value,
                remote: self.remote.take().unwrap(),
            }))
        } else if self.remote.as_ref().unwrap().alive() {
            Ok(Async::NotReady)
        } else {
            Err(Error::ConnectionDropped)
        }
    }
}

impl<T: for<'a> Reflect<'a>, R: Remote> Client<T, R> {
    pub fn new(remote: R) -> impl Future<Item=Client<T,R>, Error=Error> {
        Connect { remote: Some(remote), ph: PhantomData }
    }

    pub fn alive(&self) -> bool {
        self.remote.alive()
    }

    pub fn update(&mut self) {
        for message in self.remote.iter() {
            self.value.command_str((), message.as_str()).expect("Invalid message received");
        }
    }

    pub fn command(&mut self, cmd: &str) -> Result<(), Error> {
        self.remote.send(cmd)
    }
}
