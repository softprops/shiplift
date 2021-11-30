//! Representations of various client errors

use hyper::{self, http, StatusCode};
use serde_json::Error as SerdeError;
use std::string::FromUtf8Error;

use futures_util::io::Error as IoError;

/// Represents the result of all docker operations
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SerdeJsonError(#[from] SerdeError),

    #[error(transparent)]
    Hyper(#[from] hyper::Error),

    #[error(transparent)]
    Http(#[from] hyper::http::Error),

    #[allow(clippy::upper_case_acronyms)]
    #[error(transparent)]
    IO(#[from] IoError),

    #[error(transparent)]
    Encoding(#[from] FromUtf8Error),

    #[error("Response doesn't have the expected format: {0}")]
    InvalidResponse(String),

    #[error("{code}: {message}")]
    Fault {
        code: StatusCode,
        message: String,
    },

    #[error("expected the docker host to upgrade the HTTP connection but it did not")]
    ConnectionNotUpgraded,
}

impl From<http::uri::InvalidUri> for Error {
    fn from(error: http::uri::InvalidUri) -> Self {
        let http_error: http::Error = error.into();
        http_error.into()
    }
}
