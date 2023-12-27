use std::{fmt, io, str::Utf8Error, string::FromUtf8Error};

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GitError {
    #[error("I/O: {0}")]
    Io(String),

    #[error("Invalid content: {0}")]
    InvalidContent(String),

    #[error("Invalid header: {0}")]
    InvalidObjectHeader(&'static str),

    #[error("Invalid object payload: {0}")]
    InvalidObjectPayload(&'static str),
}

impl From<io::Error> for GitError {
    fn from(err: io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<FromUtf8Error> for GitError {
    fn from(err: FromUtf8Error) -> Self {
        Self::InvalidContent(err.to_string())
    }
}

impl From<Utf8Error> for GitError {
    fn from(err: Utf8Error) -> Self {
        Self::InvalidContent(err.to_string())
    }
}

impl From<fmt::Error> for GitError {
    fn from(err: fmt::Error) -> Self {
        Self::InvalidContent(err.to_string())
    }
}
