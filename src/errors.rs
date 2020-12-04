//! Representations of various client errors

use std::string::FromUtf8Error;

use hyper::{self, http, StatusCode};
use serde_json::Error as SerdeError;
use thiserror::Error as ThisError;
use futures_util::io::Error as IoError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0}")]
    SerdeJsonError(#[from] SerdeError),

    #[error("{0}")]
    Hyper(#[from] hyper::Error),

    #[error("{0}")]
    Http(#[from] hyper::http::Error),

    #[error("{0}")]
    IO(#[from] IoError),

    #[error("{0}")]
    Encoding(#[from] FromUtf8Error),

    #[error("Response doesn't have the expected format: {0}")]
    InvalidResponse(String),

    #[error("{code}")]
    Fault { code: StatusCode, message: String },

    #[error("expected the docker host to upgrade the HTTP connection but it did not")]
    ConnectionNotUpgraded,
}

impl From<http::uri::InvalidUri> for Error {
    fn from(error: http::uri::InvalidUri) -> Self {
        let http_error: http::Error = error.into();
        http_error.into()
    }
}

