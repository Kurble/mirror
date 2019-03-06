#[derive(Debug)]
pub enum Error {
    Json(serde_json::Error),

    ParseIntError(std::num::ParseIntError),

    Command(String),

    WrongArgumentCount,

    PathError,

    InvalidCommand,

    IncompatibleCommand,

    ConnectionDropped,
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParseIntError(err)
    }
}