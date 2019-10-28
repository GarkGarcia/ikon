use crate::{AsSize, ResampleError};
use std::{
    convert::From,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    io
};

/// The error type for operations of the `Encode` trait.
pub enum EncodingError<K: AsSize + Send + Sync> {
    /// The icon already includes an entry associated with this key.
    AlreadyIncluded(K),
    /// A resampling error.
    Resample(ResampleError),
}

impl<K: AsSize + Send + Sync> EncodingError<K> {
    /// Converts `self` to a `EncodingError<T>` using `f`.
    pub fn map<T: AsSize + Send + Sync, F: FnOnce(K) -> T>(
        self,
        f: F
    ) -> EncodingError<T> {
        match self {
            Self::AlreadyIncluded(e) => EncodingError::AlreadyIncluded(f(e)),
            Self::Resample(err) => EncodingError::Resample(err),
        }
    }
}

impl<K: AsSize + Send + Sync> Display for EncodingError<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyIncluded(_) => write!(
                f,
                "the Encode already contains an entry associated with this key"
            ),
            Self::Resample(err) => <ResampleError as Display>::fmt(&err, f),
        }
    }
}

impl<K: AsSize + Send + Sync + Debug> Debug for EncodingError<K> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::AlreadyIncluded(e) => write!(f, "Error::AlreadyIncluded({:?})", e),
            Self::Resample(err) => <ResampleError as Debug>::fmt(&err, f),
        }
    }
}

impl<K: AsSize + Send + Sync + Debug> Error for EncodingError<K> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Resample(ref err) = self {
            err.source()
        } else {
            None
        }
    }
}

impl<K: AsSize + Send + Sync> From<ResampleError> for EncodingError<K> {
    fn from(err: ResampleError) -> Self {
        Self::Resample(err)
    }
}

impl<K: AsSize + Send + Sync> From<io::Error> for EncodingError<K> {
    fn from(err: io::Error) -> Self {
        Self::from(ResampleError::from(err))
    }
}

impl<K: AsSize + Send + Sync> Into<io::Error> for EncodingError<K> {
    fn into(self) -> io::Error {
        if let Self::Resample(err) = self {
            err.into()
        } else {
            io::Error::from(io::ErrorKind::InvalidInput)
        }
    }
}
