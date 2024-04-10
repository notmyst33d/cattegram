use std::error;
use std::fmt;

#[derive(Debug)]
pub enum TlError {
    NotEnoughData,
    NoReader,
}

impl fmt::Display for TlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TlError::NotEnoughData => write!(f, "Not enough data"),
            TlError::NoReader => write!(f, "No reader"),
        }
    }
}

impl error::Error for TlError {}
