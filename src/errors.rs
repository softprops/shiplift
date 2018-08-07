//! Representations of various client errors

use http;
use hyper::{self, StatusCode};
use rustc_serialize::json::{DecoderError, EncoderError, ParserError};
use std::error::Error as ErrorTrait;
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    Decoding(DecoderError),
    Encoding(EncoderError),
    Parse(ParserError),
    Hyper(hyper::Error),
    Http(http::Error),
    IO(IoError),
    Fault { code: StatusCode, message: String },
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
            &Error::Decoding(ref err) => return err.fmt(f),
            &Error::Encoding(ref err) => return err.fmt(f),
            &Error::Parse(ref err) => return err.fmt(f),
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
            &Error::Decoding(ref err) => Some(err),
            &Error::Encoding(ref err) => Some(err),
            &Error::Parse(ref err) => Some(err),
            &Error::Http(ref err) => Some(err),
            &Error::IO(ref err) => Some(err),
            _ => None,
        }
    }
}
