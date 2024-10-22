use std::{io, fmt::{self, Display, Formatter}, error::Error};

macro_rules! description {
    ($err : expr) => ( <String as AsRef<str>>::as_ref(&format!("{}", $err)) );
}

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
            Self::Io(err) => {
                Self::Io(io::Error::new(err.kind(), description!(err)))
            },
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

impl From<DecodingError> for io::Error {
    fn from(err: DecodingError) -> io::Error {
        match err {
            DecodingError::Io(err) => err,
            DecodingError::Unsupported(msg) => {
                io::Error::new(io::ErrorKind::InvalidInput, msg)
            }
        }
    }
}
