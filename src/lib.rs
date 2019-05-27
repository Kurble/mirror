extern crate serde;
extern crate serde_json;
extern crate mirror_derive;

pub mod error;
pub mod list;
pub mod map;
pub mod primitive;
pub mod option;
pub mod hidden;

pub mod remote;
pub mod client;
mod reply;
pub mod private_server;
pub mod shared_server;

pub use mirror_derive::*;

pub use self::error::*;
pub use self::list::*;
pub use self::map::*;
pub use self::primitive::*;
pub use self::option::*;
pub use self::hidden::*;
pub use self::remote::*;
pub use self::client::*;
pub use self::private_server::*;
pub use self::shared_server::*;

use serde::*;
use serde_json::{Value, StreamDeserializer};
use serde_json::de::StrRead;

/// A single command that can be executed on a `Reflect` object.
/// Commands can be wrapped inside zero or more `Command::Path` elements that tell `Reflect`
/// implementations how to navigate to their children.
/// Each variant has an associated syntax that you can use when calling `Command::parse()`.
/// The bits that are between {} represent either another command {sub command} or json {x-json}.
#[derive(Debug)]
pub enum Command {
    /// Traverse to the child element denoted by `element`.
    /// After traversing, execute `command` on the child.
    /// syntax: `element/{sub command}`
    Path {
        element: String,
        command: Box<Command>,
    },

    /// Overwrite the entire value of the current element by deserializing `value`.
    /// syntax: `set:{value-json}`
    Set {
        value: Value,
    },

    /// Push a new element deserialized from `value` to a list.
    /// Requires the current element to be a list (`Vec`, `VecDeque`, etc.).
    /// syntax: `push:{value-json}`
    Push {
        value: Value,
    },

    /// Pop an element from the end of a list.
    /// Requires the current element to be a list (`Vec`, `VecDeque`, etc.).
    /// syntax: `pop:`
    Pop,

    /// Insert an element deserialized from `value` in a container
    ///  using the key deserialized from `key`.
    /// Requires the current element to be a list or map.
    /// syntax: `insert:{key-json} {value-json}`
    Insert {
        key: Value,
        value: Value,
    },

    /// Remove an element from a container using the key deserialized from `key`.
    /// Requires the current element to be a list or map.
    /// syntax: `remove:{key-json}`
    Remove {
        key: Value,
    },

    /// Call a function on the current element.
    /// This requires the current element to export the desired function using the #ReflectFn(..)
    /// proc-macro.
    /// The number of json arguments should match the argument count of the function.
    /// The `Context` argument does not count.
    /// syntax: `call:{name}:{arg0-json} {arg1-json} {arg2-json}...`
    Call {
        key: String,
        arguments: Vec<Value>,
    }
}

/// Context for executing commands
pub trait Context {
    type Inner: Context;

    /// Immediately run a command on the provided value. This value should be self.
    /// If the context is within a network, it will also schedule a message to relevant `Remote`s.
    fn command<R: Reflect, S: AsRef<str>>(&mut self, value: &mut R, cmd: S) -> Result<(), Error>;

    /// Same as command(..), but the scheduled message is *not* sent to the `Remote` that 
    ///  created this `Context`.
    fn local_command<R: Reflect, S: AsRef<str>>(&mut self, value: &mut R, cmd: S) -> Result<(), Error>;

    /// Take the context a level deeper. This is used by `Reflect` when traversing a path.
    /// Network contexts can use this to keep track of the root
    fn with_inner<F: FnMut(Self::Inner)>(self, path: &str, f: F);
}

/// Trait for executing commands
pub trait Reflect: for<'de> Deserialize<'de> {
    /// Executes the command on this object. If the command is executed successfully, Ok will be
    /// returned. Otherwise, an Err with the error will be returned.
    fn command<C: Context>(&mut self, context: C, command: &Command) -> Result<(), Error>;

    /// Same as `command`, but with a command as `&str` that still has to be parsed
    fn command_str<C: Context>(&mut self, context: C, command: &str) -> Result<(), Error> {
        Ok(self.command(context, &Command::parse(command)?)?)
    }
}

/// Dummy context for when no context is needed
impl Context for () {
    type Inner = ();

    fn command<R, S>(&mut self, value: &mut R, command: S) -> Result<(), Error> where
        R: Reflect,
        S: AsRef<str>
    {
        value.command_str((), command.as_ref())
    }

    fn local_command<R, S>(&mut self, value: &mut R, command: S) -> Result<(), Error> where
        R: Reflect,
        S: AsRef<str>
    {
        value.command_str((), command.as_ref())
    }

    fn with_inner<F: FnMut(Self::Inner)>(self, _: &str, mut f: F) { f(self) }
}

impl Command {
    /// Parse a `Command` from a `&str`.
    /// If there is an error during parsing, an Err will be returned.
    /// If the command is valid, Ok will be returned.
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
        fn set_bar<C: Context>(&mut self, _context: C, bar: usize, bar2: String) {
            println!("Bar should be set to {} with bar2 {}", bar, bar2);
            //self.bar = bar;
        }
    }

    #[test]
    fn it_works() {
        let mut test = FooBar { foo: Foo { bar: vec![0, 1, 2] } };
        test.command_str((), "foo/bar/set:[128, 129, 130]").unwrap();
        test.command_str((), "foo/bar/1/set:5").unwrap();
        test.command_str((), "foo/call:set_bar:16 \"test\"").unwrap();
        assert_eq!(test.foo.bar[1], 5);

        let mut test = String::from("test");
        test.command_str((), "set:\"foo bar\"").unwrap();
        assert_eq!(test, String::from("foo bar"));
    }
}
