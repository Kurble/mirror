extern crate serde;
extern crate serde_json;
extern crate regex;
extern crate mirror_derive;

pub mod error;
pub mod list;
pub mod primitive;
pub mod hidden;

pub use mirror_derive::*;

pub use self::error::*;
pub use self::list::*;
pub use self::primitive::*;
pub use self::hidden::*;

use regex::{Regex, RegexSet};
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
        let set = &[
            r"[A-Za-z_0-9]*/",
            r"set:",
            r"push:",
            r"pop:",
            r"insert:",
            r"remove:",
            r"call:",
        ];

        let commands = RegexSet::new(set)?;

        if commands.is_match(command) {
            let matches = commands.matches(command);
            let index = matches.into_iter().next().unwrap();
            let regex = Regex::new(set[index])?;
            let split_command = command.split_at(regex.shortest_match(command).unwrap());

            match index {
                0 => {
                    Ok(Command::Path {
                        element: split_command.0.split('/').next().unwrap().into(),
                        command: Box::new(Command::parse(split_command.1)?),
                    })
                },
                1 => {
                    let mut stream = StreamDeserializer::new(StrRead::new(split_command.1));
                    Ok(Command::Set {
                        value: stream.next().ok_or(Error::WrongArgumentCount)??,
                    })
                },
                2 => {
                    let mut stream = StreamDeserializer::new(StrRead::new(split_command.1));
                    Ok(Command::Push {
                        value: stream.next().ok_or(Error::WrongArgumentCount)??,
                    })
                },
                3 => {
                    Ok(Command::Pop)
                },
                4 => {
                    let mut stream = StreamDeserializer::new(StrRead::new(split_command.1));
                    Ok(Command::Insert {
                        key: stream.next().ok_or(Error::WrongArgumentCount)??,
                        value: stream.next().ok_or(Error::WrongArgumentCount)??,
                    })
                },
                5 => {
                    let mut stream = StreamDeserializer::new(StrRead::new(split_command.1));
                    Ok(Command::Remove {
                        key: stream.next().ok_or(Error::WrongArgumentCount)??,
                    })
                },
                6 => {
                    let cmd = split_command.1;
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
                }
                _ => {
                    unimplemented!()
                },
            }
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
