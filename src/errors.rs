//! Representations of various client errors

use http;
use hyper::{self, StatusCode};
use serde_json::Error as SerdeError;
use std::error::Error as ErrorTrait;
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    SerdeJsonError(SerdeError),
    Hyper(hyper::Error),
    Http(http::Error),
    IO(IoError),
    Fault { code: StatusCode, message: String },
}

impl From<SerdeError> for Error {
    fn from(error: SerdeError) -> Error {
        Error::SerdeJsonError(error)
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error::Hyper(error)
    }
}

impl From<http::Error> for Error {
    fn from(error: http::Error) -> Error {
        Error::Http(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::IO(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Docker Error: ")?;
        match self {
            &Error::SerdeJsonError(ref err) => return err.fmt(f),
            &Error::Http(ref err) => return err.fmt(f),
            &Error::Hyper(ref err) => return err.fmt(f),
            &Error::IO(ref err) => return err.fmt(f),
            &Error::Fault { code, .. } => return write!(f, "{}", code),
        };
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        "Shiplift Error"
    }

    fn cause(&self) -> Option<&ErrorTrait> {
        match self {
            &Error::SerdeJsonError(ref err) => Some(err),
            &Error::Http(ref err) => Some(err),
            &Error::IO(ref err) => Some(err),
            _ => None,
        }
    }
}
