use std::str;
use std::{error, fmt, io};

use futures::sync::{mpsc, oneshot};

use codec;

#[derive(Debug)]
pub enum Error {
    /// A Non-Specific internal error than prevented and operation from completing
    Internal(String),

    /// An IO error
    IO(io::Error),

    /// An parsing/serialising error occurred
    Value(String, Option<codec::NSQValue>),

    /// An critical Unexpected Error
    Unexpected(String),

    /// End of Stream connection is broken
    EndOfStream,
}

pub fn internal<T: Into<String>>(msg: T) -> Error {
    Error::Internal(msg.into())
}

pub fn value<T: Into<String>>(msg: T, val: codec::NSQValue) -> Error {
    Error::Value(msg.into(), Some(val))
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

impl From<oneshot::Canceled> for Error {
    fn from(err: oneshot::Canceled) -> Error {
        Error::Unexpected(format!("Oneshot was cancelled before use: {}", err))
    }
}

impl<T: 'static + Send> From<mpsc::SendError<T>> for Error {
    fn from(err: mpsc::SendError<T>) -> Error {
        Error::Unexpected(format!("Cannot write to channel: {}", err))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IO(ref err) => err.description(),
            Error::Value(ref s, _) => s,
            Error::Unexpected(ref s) => s,
            Error::Internal(ref s) => s,
            Error::EndOfStream => "End of Stream",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IO(ref err) => Some(err),
            Error::Value(_, _) => None,
            Error::Internal(_) => None,
            Error::Unexpected(_) => None,
            Error::EndOfStream => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        fmt::Display::fmt(self.description(), f)
    }
}
