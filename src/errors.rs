//! Representations of various client errors

use hyper::Error as HttpError;
use hyper::status::StatusCode;
use rustc_serialize::json::{DecoderError, EncoderError, ParserError};
use std::error::Error as ErrorTrait;
use std::fmt;
use std::io::Error as IoError;

#[cfg(feature = "ssl")]
use openssl::error::ErrorStack;

#[derive(Debug)]
pub enum Error {
    Decoding(DecoderError),
    Encoding(EncoderError),
    Parse(ParserError),
    Http(HttpError),
    IO(IoError),
    Fault { code: StatusCode, message: String },
    Message(String),
    #[cfg(feature = "ssl")]
    Stack(ErrorStack),
}

impl From<ParserError> for Error {
    fn from(error: ParserError) -> Error {
        Error::Parse(error)
    }
}

impl From<DecoderError> for Error {
    fn from(error: DecoderError) -> Error {
        Error::Decoding(error)
    }
}

impl From<EncoderError> for Error {
    fn from(error: EncoderError) -> Error {
        Error::Encoding(error)
    }
}

impl From<HttpError> for Error {
    fn from(error: HttpError) -> Error {
        Error::Http(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::IO(error)
    }
}

impl From<String> for Error {
    fn from(error: String) -> Error {
        Error::Message(error)
    }
}

#[cfg(feature = "ssl")]
impl From<ErrorStack> for Error {
    fn from(error: ErrorStack) -> Error {
        Error::Stack(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Docker Error: ")?;
        match self {
            &Error::Decoding(ref err) => return err.fmt(f),
            &Error::Encoding(ref err) => return err.fmt(f),
            &Error::Parse(ref err) => return err.fmt(f),
            &Error::Http(ref err) => return err.fmt(f),
            &Error::IO(ref err) => return err.fmt(f),
            &Error::Fault { code, .. } => return write!(f, "{}", code),
            &Error::Message(ref err) => return err.fmt(f),

            #[cfg(feature = "ssl")]
            &Error::Stack(ref err) => return err.fmt(f),
        };
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        "Shiplift Error"
    }

    fn cause(&self) -> Option<&ErrorTrait> {
        match self {
            &Error::Decoding(ref err) => Some(err),
            &Error::Encoding(ref err) => Some(err),
            &Error::Parse(ref err) => Some(err),
            &Error::Http(ref err) => Some(err),
            &Error::IO(ref err) => Some(err),
            _ => None,
        }
    }
}
