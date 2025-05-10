use std::{fmt::Display, io};

use io_uring::squeue::PushError;

use crate::pstatus::PromiseStatus;

#[derive(Debug)]
pub enum Error {
    Push,
    IO(io::Error),
    Promise(PromiseStatus)
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

impl From<PromiseStatus> for Error {
    fn from(status: PromiseStatus) -> Self {
        Self::Promise(status)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Push => write!(f, "submission queue is full"),
            Error::IO(error) => write!(f, "io error: {error}"),
            Error::Promise(status) => write!(f, "promise not ready; has status {status}")
        }
    }
}

impl core::error::Error for Error {}