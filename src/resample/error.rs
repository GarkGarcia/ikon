use std::{
    convert::From,
    error::Error,
    fmt::{self, Display},
    io
};

const MISMATCHED_DIM_ERR: &str =
    "a resampling filter returned an image of dimensions other than the ones specified by it's arguments";

#[derive(Debug)]
/// The error type for resampling operations.
pub enum ResampleError {
    /// Generic I/O error.
    Io(io::Error),
    /// A resampling filter produced results of dimensions
    /// other the ones specified by it's arguments.
    MismatchedDimensions(u32, (u32, u32)),
}

impl From<io::Error> for ResampleError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl Display for ResampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{}", err),
            Self::MismatchedDimensions(s, (w, h)) => write!(
                f,
                "{0}: expected {1}x{1}, got {2}x{3}",
                MISMATCHED_DIM_ERR, s, w, h
            ),
        }
    }
}

impl Error for ResampleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Io(ref err) = self {
            Some(err)
        } else {
            None
        }
    }
}

impl Into<io::Error> for ResampleError {
    fn into(self) -> io::Error {
        match self {
            Self::Io(err) => err,
            Self::MismatchedDimensions(_, _) => io::Error::from(io::ErrorKind::InvalidData),
        }
    }
}
