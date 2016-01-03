use std::io::Error as IoError;
use hyper::Error as HttpError;
use hyper::status::StatusCode;
use rustc_serialize::json::{DecoderError,EncoderError};

#[derive(Debug)]
pub enum Error {
    Decoding(DecoderError),
    Encoding(EncoderError),
    Http(HttpError),
    IO(IoError),
    Fault { code: StatusCode, message: String }
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
