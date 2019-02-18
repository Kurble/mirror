#[derive(Debug)]
pub enum Error {
    Regex(regex::Error),

    Json(serde_json::Error),

    ParseIntError(std::num::ParseIntError),

    Command(String),

    WrongArgumentCount,

    PathError,

    InvalidCommand,

    IncompatibleCommand,
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::Regex(err)
    }
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