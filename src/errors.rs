//! Representations of various client errors

use http;
use hyper::{self, StatusCode};
use serde_json::Error as SerdeError;
use std::{error::Error as StdError, fmt, io::Error as IoError, string::FromUtf8Error};

#[derive(Debug)]
pub enum Error {
    SerdeJsonError(SerdeError),
    Hyper(hyper::Error),
    Http(http::Error),
    IO(IoError),
    Encoding(FromUtf8Error),
    InvalidResponse(String),
    Fault { code: StatusCode, message: String },
    ConnectionNotUpgraded,
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
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "Docker Error: ")?;
        match self {
            Error::SerdeJsonError(ref err) => err.fmt(f),
            Error::Http(ref err) => err.fmt(f),
            Error::Hyper(ref err) => err.fmt(f),
            Error::IO(ref err) => err.fmt(f),
            Error::Encoding(ref err) => err.fmt(f),
            Error::InvalidResponse(ref cause) => {
                write!(f, "Response doesn't have the expected format: {}", cause)
            }
            Error::Fault { code, .. } => write!(f, "{}", code),
            Error::ConnectionNotUpgraded => write!(
                f,
                "expected the docker host to upgrade the HTTP connection but it did not"
            ),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::SerdeJsonError(e) => e.description(),
            Error::Hyper(e) => e.description(),
            Error::Http(e) => e.description(),
            Error::IO(e) => e.description(),
            Error::Encoding(e) => e.description(),
            Error::InvalidResponse(msg) => msg.as_str(),
            Error::Fault { message, .. } => message.as_str(),
            Error::ConnectionNotUpgraded => "connection not upgraded",
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match self {
            Error::SerdeJsonError(ref err) => Some(err),
            Error::Http(ref err) => Some(err),
            Error::IO(ref err) => Some(err),
            Error::Encoding(e) => Some(e),
            _ => None,
        }
    }
}
