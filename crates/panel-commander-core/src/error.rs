use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Protocol(String),
    InvalidEdid(String),
    Unsupported(String),
    NotFound(String),
    Permission(String),
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Protocol(s) => write!(f, "DDC/CI protocol error: {s}"),
            Error::InvalidEdid(s) => write!(f, "invalid EDID: {s}"),
            Error::Unsupported(s) => write!(f, "unsupported: {s}"),
            Error::NotFound(s) => write!(f, "not found: {s}"),
            Error::Permission(s) => write!(f, "permission denied: {s}"),
            Error::InvalidArgument(s) => write!(f, "invalid argument: {s}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}
