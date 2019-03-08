use super::*;
use crate::reply::Reply;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::Receiver;
use serde::Serialize;

pub struct SharedServer<T: for<'a> Reflect<'a> + Serialize, R: Remote> {
    value: T,
    listener: Receiver<R>,
    clients: Vec<R>,
}

impl<T: for<'a> Reflect<'a> + Serialize, R: Remote> Deref for SharedServer<T, R> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: for<'a> Reflect<'a> + Serialize, R: Remote> DerefMut for SharedServer<T, R> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: for<'a> Reflect<'a> + Serialize, R: Remote> SharedServer<T, R> {
    pub fn new(value: T, listener: Receiver<R>) -> Self {
        Self {
            value,
            listener,
            clients: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        for mut new_client in self.listener.try_iter() {
            if new_client.send(serde_json::to_string(&self.value).unwrap().as_str()).is_ok() {
                self.clients.push(new_client);
            }
        }

        for client_id in 0..self.clients.len() {
            let mut failed = false;
            let mut reply = Vec::new();

            for message in self.clients[client_id].iter() {
                if self.value.command_str(Reply::new(&mut reply), message.as_str()).is_err() {
                    failed = true;
                    break;
                }
            }

            for client in self.clients.iter_mut() {
                for msg in reply.iter() {
                    if client.send(msg.as_str()).is_err() {
                        client.close();
                    }
                }
            }

            if failed {
                self.clients[client_id].close();
            }
        }

        self.clients.retain(|c| c.alive());
    }

    pub fn local_command(&mut self, cmd: &str) -> Result<(), Error> {
        let mut reply = Vec::new();
        self.value.command_str(Reply::new(&mut reply), cmd)?;

        for client in self.clients.iter_mut() {
            for msg in reply.iter() {
                if client.send(msg.as_str()).is_err() {
                    client.close();
                }
            }
        }

        Ok(())
    }

    pub fn command(&mut self, cmd: &str) -> Result<(), Error> {
        self.value.command_str((), cmd)?;

        for client in self.clients.iter_mut() {
            if client.send(cmd).is_err() {
                client.close();
            }
        }

        Ok(())
    }

    pub fn clients(&self) -> usize {
        self.clients.len()
    }
}

