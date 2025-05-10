use std::{fmt::Display, io};

use io_uring::squeue::PushError;

#[derive(Debug)]
pub enum Error {
    Push,
    IO(io::Error),
}

impl From<PushError> for Error {
    fn from(_: PushError) -> Self {
        Self::Push
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IO(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Push => write!(f, "submission queue is full"),
            Error::IO(error) => write!(f, "io error: {error}"),
        }
    }
}

impl core::error::Error for Error {}