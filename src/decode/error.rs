use std::{io, fmt::{self, Display, Formatter}, error::Error};

#[derive(Debug)]
/// The error type for operations of the `Decode` trait.
pub enum DecodingError {
    /// A generic IO error.
    Io(io::Error),
    /// The decoder does not support a particular feature
    /// present in it's input.
    Unsupported(String)
}

impl Clone for DecodingError {
    fn clone(&self) -> Self {
        match self {
            Self::Io(err) => Self::Io(io::Error::new(err.kind(), err.description())),
            Self::Unsupported(msg) => Self::Unsupported(msg.clone())
        }
    }
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Unsupported(msg) => write!(f, "{}", msg)
        }
    }
}

impl Error for DecodingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None
        }
    }
}

impl From<io::Error> for DecodingError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl Into<io::Error> for DecodingError {
    fn into(self) -> io::Error {
        match self {
            Self::Io(err) => err,
            Self::Unsupported(msg) => io::Error::new(io::ErrorKind::InvalidInput, msg)
        }
    }
}