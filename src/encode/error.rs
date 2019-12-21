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
    /// The icon aready stores the maximum number of entries possible.
    Full(u16)
}

impl<K: AsSize + Send + Sync> Display for EncodingError<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyIncluded(_) => write!(
                f,
                "The Encode already contains an entry associated with this key"
            ),
            Self::Resample(err) => <ResampleError as Display>::fmt(&err, f),
            Self::Full(max_n) => write!(
                f,
                "The icon has already reached it's maximum capacity ({} entries)",
                max_n
            )
        }
    }
}

impl<K: AsSize + Send + Sync + Debug> Debug for EncodingError<K> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::AlreadyIncluded(e) => write!(
                f,
                "EncodingError::AlreadyIncluded({:?})",
                e
            ),
            Self::Resample(err) => write!(f, "EncodingError::Resample({:?})", err),
            Self::Full(n) => write!(f, "EncodingError::Full({})", n)
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
            io::Error::new(io::ErrorKind::InvalidInput, format!("{}", self))
        }
    }
}
