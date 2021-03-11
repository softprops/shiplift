//! Representations of various client errors

use hyper::{self, http, StatusCode};
use serde_json::Error as SerdeError;
use std::{error::Error as StdError, fmt, string::FromUtf8Error};

use futures_util::io::Error as IoError;

/// Represents the result of all docker operations
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "tls")]
use openssl::error::ErrorStack;

#[derive(Debug)]
pub enum Error {
    SerdeJsonError(SerdeError),
    Hyper(hyper::Error),
    Http(hyper::http::Error),
    IO(IoError),
    Encoding(FromUtf8Error),
    InvalidResponse(String),
    Fault {
        code: StatusCode,
        message: String,
    },
    ConnectionNotUpgraded,
    #[cfg(feature = "tls")]
    ErrorStack(ErrorStack),
    UnsupportedScheme(String),
    MissingAuthority,
}

#[cfg(feature = "tls")]
impl From<ErrorStack> for Error {
    fn from(error: ErrorStack) -> Error {
        Error::ErrorStack(error)
    }
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
            #[cfg(feature = "tls")]
            Error::ErrorStack(stack) => {
                writeln!(f, "Error stack:")?;
                for error in stack.errors() {
                    writeln!(f, " - {}", error)?;
                }
                Ok(())
            }
            Error::UnsupportedScheme(scheme) => {
                if scheme.as_str() == "unix" {
                    writeln!(f, "Provided scheme `unix` is not supported. Recompile with `unix-socket` feature enabled.")
                } else {
                    writeln!(f, "Provided scheme `{}` is not supported", scheme)
                }
            }
            Error::MissingAuthority => {
                writeln!(f, "Provided URI is missing authority part after scheme")
            }
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
            _ => None,
        }
    }
}
