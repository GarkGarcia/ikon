use crate::{Icon, ResampleError};
use std::{
    convert::From,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    io
};

/// The error type for operations of the `Encode` trait.
pub enum EncodingError<I: Icon + Send + Sync> {
    /// The icon family already includes this icon.
    AlreadyIncluded(I),
    /// A resampling error.
    Resample(ResampleError),
    /// The icon family aready stores the maximum number of icons possible.
    Full(u16)
}

impl<I: Icon + Send + Sync> Display for EncodingError<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyIncluded(_) => write!(
                f, "The icon family already contains this icon"
            ),
            Self::Resample(err) => <ResampleError as Display>::fmt(&err, f),
            Self::Full(max_n) => write!(
                f,
                "The icon family has already reached it's maximum capacity ({} icons)",
                max_n
            )
        }
    }
}

impl<I: Icon + Send + Sync + Debug> Debug for EncodingError<I> {
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

impl<I: Icon + Send + Sync + Debug> Error for EncodingError<I> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Resample(ref err) = self {
            err.source()
        } else {
            None
        }
    }
}

impl<I: Icon + Send + Sync> From<ResampleError> for EncodingError<I> {
    fn from(err: ResampleError) -> Self {
        Self::Resample(err)
    }
}

impl<I: Icon + Send + Sync> From<io::Error> for EncodingError<I> {
    fn from(err: io::Error) -> Self {
        Self::from(ResampleError::from(err))
    }
}

impl<I: Icon + Send + Sync> Into<io::Error> for EncodingError<I> {
    fn into(self) -> io::Error {
        if let Self::Resample(err) = self {
            err.into()
        } else {
            io::Error::new(io::ErrorKind::InvalidInput, format!("{}", self))
        }
    }
}

