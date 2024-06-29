use std::error;
use std::fmt;

#[derive(Debug)]
pub enum TlError {
    NotEnoughData,
    NoReader(i32),
}

impl fmt::Display for TlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TlError::NotEnoughData => write!(f, "Not enough data"),
            TlError::NoReader(hash) => write!(f, "No reader for {}", hash),
        }
    }
}

impl error::Error for TlError {}
