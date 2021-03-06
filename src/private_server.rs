use super::*;
use crate::reply::Reply;

use std::ops::Deref;
use std::sync::mpsc::Receiver;

pub struct PrivateClient<T: Reflect + Serialize, R: Remote> {
    value: T,
    remote: R,
}

pub struct PrivateServer<T: Reflect + Serialize, R: Remote> {
    factory: Box<Fn() -> T>,
    listener: Receiver<R>,
    clients: Vec<PrivateClient<T, R>>,
}

impl<T: Reflect + Serialize, R: Remote> Deref for PrivateClient<T, R> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: Reflect + Serialize, R: Remote> PrivateServer<T, R> {
    pub fn new<F: 'static + Fn()->T>(factory: F, listener: Receiver<R>) -> Self {
        Self {
            factory: Box::new(factory),
            listener,
            clients: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        for mut remote in self.listener.try_iter() {
            let value = (self.factory)();

            // send over the base value to the remote as part of the protocol
            if remote.send(serde_json::to_string(&value).unwrap().as_str()).is_ok() {
                self.clients.push(PrivateClient { value, remote });
            }
        }

        for client in self.clients.iter_mut() {
            let mut failed = false;
            let reply = Reply::new(Vec::new());

            for message in client.remote.iter() {
                if client.value.command_str(reply.clone(), message.as_str()).is_err() {
                    failed = true;
                }
            }

            for (r, send) in reply.into_inner().into_iter() {
                if send {
                    failed &= client.remote.send(r.as_str()).is_err();
                }
            }

            if failed {
                client.remote.close();
            }
        }

        self.clients.retain(|c| c.remote.alive());
    }

    pub fn clients(&mut self) -> impl Iterator<Item = &mut PrivateClient<T, R>> {
        self.clients.iter_mut()
    }
}

impl<T: Reflect + Serialize, R: Remote> PrivateClient<T, R> {
    pub fn command(&mut self, command: &str) -> Result<(), Error> {
        match self.value.command_str((), command) {
            Ok(_) => {
                self.remote.send(command)?;
                Ok(())
            },

            Err(e) => {
                self.remote.close();
                Err(e)
            },
        }
    }

    pub fn kick(&mut self) {
        self.remote.close();
    }
}
