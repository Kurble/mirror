extern crate serde;
extern crate serde_json;
extern crate mirror_derive;

pub mod error;
pub mod list;
pub mod primitive;
pub mod hidden;

pub mod remote;
pub mod client;
pub mod private_server;
pub mod shared_server;

pub use mirror_derive::*;

pub use self::error::*;
pub use self::list::*;
pub use self::primitive::*;
pub use self::hidden::*;
pub use self::remote::*;

use serde::*;
use serde_json::{Value, StreamDeserializer};
use serde_json::de::StrRead;

#[derive(Debug)]
pub enum Command {
    // "element/{command}"
    Path {
        element: String,
        command: Box<Command>,
    },

    // valid for things that can be deserialized "set:{json}"
    Set {
        value: Value,
    },

    // valid for things that are a list. "push:{json}"
    Push {
        value: Value,
    },

    // remove last element from a list. "pop"
    Pop,

    // valid for things that are a map. "put:{json} {json}"
    Insert {
        key: Value,
        value: Value,
    },

    // valid for things that are a map. "remove:{json}"
    Remove {
        key: Value,
    },

    // valid for functions. "call:name:{json} {json} {json}...
    Call {
        key: String,
        arguments: Vec<Value>,
    }
}

pub trait Reflect<'de>: Deserialize<'de> {
    fn command(&mut self, command: &Command) -> Result<(), Error>;

    fn command_str(&mut self, cmd: &str) -> Result<(), Error> {
        Ok(self.command(&Command::parse(cmd)?)?)
    }
}

impl Command {
    pub fn parse(command: &str) -> Result<Self, Error> {
        if command.starts_with("set:") {
            let mut stream = StreamDeserializer::new(StrRead::new(command.split_at("set:".len()).1));
            Ok(Command::Set {
                value: stream.next().ok_or(Error::WrongArgumentCount)??,
            })
        } else if command.starts_with("push:") {
            let mut stream = StreamDeserializer::new(StrRead::new(command.split_at("push:".len()).1));
            Ok(Command::Push {
                value: stream.next().ok_or(Error::WrongArgumentCount)??,
            })
        } else if command.starts_with("pop:") {
            Ok(Command::Pop)
        } else if command.starts_with("insert:") {
            let mut stream = StreamDeserializer::new(StrRead::new(command.split_at("insert:".len()).1));
            Ok(Command::Insert {
                key: stream.next().ok_or(Error::WrongArgumentCount)??,
                value: stream.next().ok_or(Error::WrongArgumentCount)??,
            })
        } else if command.starts_with("remove:") {
            let mut stream = StreamDeserializer::new(StrRead::new(command.split_at("remove:".len()).1));
            Ok(Command::Remove {
                key: stream.next().ok_or(Error::WrongArgumentCount)??,
            })
        } else if command.starts_with("call:") {
            let cmd = command.split_at("call:".len()).1;
            let (key, args) = cmd.split_at(cmd.find(':').unwrap());

            let stream = StreamDeserializer::new(StrRead::new(&args[1..]));
            let mut arguments = Vec::new();
            for result in stream {
                arguments.push(result?);
            }

            Ok(Command::Call {
                key: key.into(),
                arguments,
            })
        } else if command.contains("/") {
            Ok(Command::Path {
                element: command.split('/').next().unwrap().into(),
                command: Box::new(Command::parse(command.split_at(command.find("/").unwrap() + 1).1)?),
            })
        } else {
            Err(Error::InvalidCommand)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, Reflect)]
    pub struct FooBar {
        foo: Foo
    }

    #[ReflectFn(
        Fn(name="set_bar", args="2")
    )]
    #[derive(Deserialize, Reflect)]
    pub struct Foo {
        bar: Vec<usize>,
    }

    impl Foo {
        fn set_bar(&mut self, bar: usize, bar2: String) {
            println!("Bar should be set to {} with bar2 {}", bar, bar2);
            //self.bar = bar;
        }
    }

    #[test]
    fn it_works() {
        let mut test = FooBar { foo: Foo { bar: vec![0, 1, 2] } };
        test.command_str("foo/bar/set:[128, 129, 130]").unwrap();
        test.command_str("foo/bar/1/set:5").unwrap();
        test.command_str("foo/call:set_bar:16 \"test\"").unwrap();
        assert_eq!(test.foo.bar[1], 5);

        let mut test = String::from("test");
        test.command_str("set:\"foo bar\"").unwrap();
        assert_eq!(test, String::from("foo bar"));
    }
}
