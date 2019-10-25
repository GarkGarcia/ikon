use crate::{AsSize};
use std::{
    convert::From,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    io
};

const MISMATCHED_DIM_ERR: &str =
    "a resampling filter returned an image of dimensions other than the ones specified by it's arguments";

/// The error type for operations of the `Icon` trait.
pub enum IconError<K: AsSize + Send + Sync> {
    /// The `Icon` instance already includes an entry associated with this key.
    AlreadyIncluded(K),
    /// A resampling error.
    Resample(ResampleError),
}

#[derive(Debug)]
/// The error type for resampling operations.
pub enum ResampleError {
    /// Generic I/O error.
    Io(io::Error),
    /// A resampling filter produced results of dimensions
    /// other the ones specified by it's arguments.
    MismatchedDimensions(u32, (u32, u32)),
}

impl<K: AsSize + Send + Sync> IconError<K> {
    /// Converts `self` to a `IconError<T>` using `f`.
    pub fn map<T: AsSize + Send + Sync, F: FnOnce(K) -> T>(
        self,
        f: F
    ) -> IconError<T> {
        match self {
            Self::AlreadyIncluded(e) => IconError::AlreadyIncluded(f(e)),
            Self::Resample(err) => IconError::Resample(err),
        }
    }
}

impl<K: AsSize + Send + Sync> Display for IconError<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyIncluded(_) => write!(
                f,
                "the icon already contains an entry associated with this key"
            ),
            Self::Resample(err) => <ResampleError as Display>::fmt(&err, f),
        }
    }
}

impl<K: AsSize + Send + Sync + Debug> Debug for IconError<K> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::AlreadyIncluded(e) => write!(f, "Error::AlreadyIncluded({:?})", e),
            Self::Resample(err) => <ResampleError as Debug>::fmt(&err, f),
        }
    }
}

impl<K: AsSize + Send + Sync + Debug> Error for IconError<K> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Resample(ref err) = self {
            err.source()
        } else {
            None
        }
    }
}

impl<K: AsSize + Send + Sync> From<ResampleError> for IconError<K> {
    fn from(err: ResampleError) -> Self {
        Self::Resample(err)
    }
}

impl<K: AsSize + Send + Sync> From<io::Error> for IconError<K> {
    fn from(err: io::Error) -> Self {
        Self::from(ResampleError::from(err))
    }
}

impl<K: AsSize + Send + Sync> Into<io::Error> for IconError<K> {
    fn into(self) -> io::Error {
        if let Self::Resample(err) = self {
            err.into()
        } else {
            io::Error::from(io::ErrorKind::InvalidInput)
        }
    }
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
