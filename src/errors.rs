//! Representations of various client errors

use hyper::{self, http, StatusCode};
use openssl::error::ErrorStack;
use serde_json::Error as SerdeError;
use std::{error::Error as StdError, fmt, string::FromUtf8Error};

use futures_util::io::Error as IoError;

/// Represents the result of all docker operations
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    SerdeJsonError(SerdeError),
    Hyper(hyper::Error),
    Http(hyper::http::Error),
    #[allow(clippy::upper_case_acronyms)]
    IO(IoError),
    Encoding(FromUtf8Error),
    InvalidResponse(String),
    Ssl(ErrorStack),
    Fault {
        code: StatusCode,
        message: String,
    },
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

impl From<hyper::http::Error> for Error {
    fn from(error: hyper::http::Error) -> Error {
        Error::Http(error)
    }
}

impl From<http::uri::InvalidUri> for Error {
    fn from(error: http::uri::InvalidUri) -> Self {
        let http_error: http::Error = error.into();
        http_error.into()
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::IO(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Error {
        Error::Encoding(error)
    }
}

impl From<ErrorStack> for Error {
    fn from(error: ErrorStack) -> Error {
        Error::Ssl(error)
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
            Error::Ssl(ref err) => write!(f, "SSL error: {}", err),
            Error::Fault { code, message } => write!(f, "{}: {}", code, message),
            Error::ConnectionNotUpgraded => write!(
                f,
                "expected the docker host to upgrade the HTTP connection but it did not"
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::SerdeJsonError(ref err) => Some(err),
            Error::Http(ref err) => Some(err),
            Error::IO(ref err) => Some(err),
            Error::Encoding(e) => Some(e),
            Error::Ssl(ref err) => Some(err),
            _ => None,
        }
    }
}
