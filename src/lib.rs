//! A simple solution for encoding common icon file-formats, such as `.ico`, `.icns` and _favicon_. 
//! 
//! This crate is mostly a wrapper for other libraries, unifying existing APIs into a single, cohesive
//! interface. It serves as **[IconPie's](https://github.com/GarkGarcia/icon-pie)** internal library.
//! 
//! # Overview
//! 
//! An _icon_ consists of a map between _keys_ and _images_. An _entry_ is a _key-value_ pair contained
//! in an _icon_.
//! 
//! **IconBaker** simply automates the process of re-scaling _images_, creating _entries_ and combining
//! them into an _icon_.
//! 
//! ## Keys
//! 
//! Each _icon_ format is associated with a particular _key type_, which determines how
//! _entries_ are labeled. Each _key_ can only be associated with a single _image_.
//! 
//! For example, _icon_ formats that only differentiate _entries_ by the dimensions of their associated
//! _images_ are labeled by _positive integers_, such as the `.ico` and `.icns` file-formats.
//! 
//! On the other hand, _icon_ formats that distinguish their _entries_ by 
//! _[path](https://en.wikipedia.org/wiki/Path_%28computing%29)_, such as _png sequeces_ and
//! _[FreeDesktop icon themes](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html)_
//! , are labeled by _path_.
//! 
//! Note that, since the dimensions
//! of the _images_ contained in an _entry_ are dictated by their associated _entries_, every _key_
//! must be convertible to a _positive integers_. Therefore, all _key types_ are required to implement
//! `AsRef<u32>`.
//! 
//! ## Resampling
//! 
//! Pictures are scaled using resampling filters, which are represented by _functions that take a source_ 
//! _image and a size and return a re-scaled image_.
//! 
//! This allows the users of this crate to provide their custom resampling filters. Common resampling 
//! filters are provided in the 
//! [`resample`](https://docs.rs/icon_baker/2.2.0/icon_baker/resample/index.html) module.
//! 
//! # Examples
//! 
//! ## General Usage
//! 
//! The `Icon::add_entry` can be used to automatically resample
//! _source images_ and converts them to _entries_ in an icon.
//! 
//! ```rust
//! use icon_baker::{ico::{Ico, Key}, SourceImage, Icon, Error};
//!   
//! fn example() -> Result<(), Error> {
//!     let icon = Ico::new();
//!     let src = SourceImage::open("image.svg")?;
//! 
//!     icon.add_entry(resample::linear, &img, Key(32))
//! }
//! ```
//! 
//! ## Writing to Disk
//! 
//! Implementors of the `Icon` trait can be written to any object
//! that implements `io::Write` with the `Icon::write` method.
//! 
//! ```rust
//! use icon_baker::favicon::Favicon;
//! use std::{io, fs::File};
//!  
//! fn example() -> io::Result<()> {
//!     let icon = Favicon::new();
//! 
//!     // Process the icon ...
//! 
//!     let file = File::create("out.icns")?;
//!     icon.write(file)
//! }
//! ```
//! 
//! Alternatively, icons can be directly written to a file on
//! disk with `Icon::save` method.
//! 
//! ```rust
//! use icon_baker::favicon::Favicon;
//! use std::{io, fs::File};
//!  
//! fn example() -> io::Result<()> {
//!     let icon = Favicon::new();
//! 
//!     /* Process the icon */
//! 
//!     icon.save("./output/")
//! }
//! ```

pub extern crate image;
pub extern crate resvg;

use crate::usvg::Tree;
use image::{DynamicImage, GenericImageView, ImageError};
pub use resvg::{raqote, usvg};
use std::{
    convert::From,
    error,
    fmt::{self, Debug, Display, Formatter},
    fs::File,
    io::{self, Write},
    path::Path,
};

pub mod favicon;
pub mod icns;
pub mod ico;
pub mod resample;
#[cfg(test)]
mod test;

const STD_CAPACITY: usize = 7;
const INVALID_DIM_ERR: &str =
    "a resampling filter returned an image of dimensions other than the ones specified by it's arguments";

/// A generic representation of an icon encoder.
pub trait Icon
where
    Self: Sized,
    Self::Key: AsSize
{
    type Key;

    /// Creates a new icon.
    ///
    /// # Example
    /// ```rust
    /// let icon = Ico::new();
    /// ```
    fn new() -> Self {
        Self::with_capacity(STD_CAPACITY)
    }

    /// Constructs a new, empty `Icon` with the specified capacity.
    /// The `capacity` argument designates the number of entries
    /// that will be allocated.
    ///
    /// # Example
    /// ```rust
    /// let icon = Ico::with_capacity(5);
    /// ```
    fn with_capacity(capacity: usize) -> Self;

    /// Returns the number of _entries_ contained in the icon.
    fn len(&self) -> usize;

    /// Adds an individual entry to the icon.
    ///
    /// # Arguments
    ///
    /// * `filter` The resampling filter that will be used to re-scale `source`.
    /// * `source` A reference to the source image this entry will be based on.
    /// * `key` Information on the target entry.
    ///
    /// # Return Value
    ///
    /// * Returns `Err(Error::InvalidDimensions(_))` if the dimensions provided in the
    ///   `key` argument are not supported.
    /// * Returns `Err(Error::AlreadyIncluded(_))` if the icon already contains
    ///   an entry associated with `key`.
    /// * Returns `Err(Error::MismatchedDimensions(_, (_, _)))`
    ///   if the resampling filter provided in the `filter` argument produces
    ///   results of dimensions other than the ones specified by `key`.
    /// * Otherwise returns `Ok(())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use icon_baker::{Ico, SourceImage, Icon, Error};
    ///  
    /// fn example() -> Result<(), Error> {
    ///     let icon = Ico::new();
    ///     let src = SourceImage::open("image.svg")?;
    ///
    ///     icon.add_entry(resample::linear, &img, 32)
    /// }
    /// ```
    fn add_entry<F: FnMut(&SourceImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        key: Self::Key,
    ) -> Result<(), Error<Self::Key>>;

    /// Adds a series of entries to the icon.
    ///
    /// # Arguments
    ///
    /// * `filter` The resampling filter that will be used to re-scale `source`.
    /// * `source` A reference to the source image this entry will be based on.
    /// * `keys` A container for the information on the target entries.
    ///
    /// # Return Value
    ///
    /// * Returns `Err(Error::InvalidDimensions(_))` if any of the items of `keys`
    ///   provides unsupported dimensions.
    /// * Returns `Err(Error::AlreadyIncluded(_))` if the icon already contains an
    ///   entry associated with any of the items of `keys`.
    /// * Returns `Err(Error::MismatchedDimensions(_, (_, _)))`
    ///   if the resampling filter provided in the `filter` argument produces
    ///   results of dimensions other than the ones specified by the items of
    ///   `keys`.
    /// * Otherwise returns `Ok(())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use icon_baker::{Icns, SourceImage, Icon, Error};
    ///  
    /// fn example() -> Result<(), Error> {
    ///     let icon = Icns::new();
    ///     let src = SourceImage::open("image.svg")?;
    ///
    ///     icon.add_entries(
    ///         resample::linear,
    ///         &src,
    ///         vec![32, 64, 128]
    ///     )
    /// }
    /// ```
    fn add_entries<
        F: FnMut(&SourceImage, u32) -> io::Result<DynamicImage>,
        I: IntoIterator<Item = Self::Key>,
    >(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        keys: I,
    ) -> Result<(), Error<Self::Key>> {
        for key in keys {
            self.add_entry(|src, size| filter(src, size), source, key)?;
        }

        Ok(())
    }

    /// Writes the contents of the icon to `w`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use icon_baker::favicon::Favicon;
    /// use std::{io, fs::File};
    ///  
    /// fn example() -> io::Result<()> {
    ///     let icon = Favicon::new();
    ///
    ///     /* Process the icon */
    ///
    ///     let file = File::create("out.icns")?;
    ///     icon.write(file)
    /// }
    /// ```
    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()>;

    /// Writes the contents of the icon to a file on disk.
    ///
    /// # Example
    ///
    /// ```rust
    /// use icon_baker::favicon::Favicon;
    /// use std::{io, fs::File};
    ///  
    /// fn example() -> io::Result<()> {
    ///     let icon = Favicon::new();
    ///
    ///     /* Process the icon */
    ///
    ///     icon.save("./output/")
    /// }
    /// ```
    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<()> {
        let mut file = File::create(path.as_ref())?;
        self.write(&mut file)
    }
}

/// A trait for types that represent the dimesions of an icon.
pub trait AsSize {
    fn as_size(&self) -> u32;
}

#[derive(Clone)]
/// A uniun type for raster and vector graphics.
pub enum SourceImage {
    /// A generic raster image.
    Raster(DynamicImage),
    /// A svg-encoded vector image.
    Svg(Tree),
}

/// The error type for operations of the `Icon` trait.
pub enum Error<K: AsSize> {
    /// The `Icon` instance already includes an entry associated with this key.
    AlreadyIncluded(K),
    /// Generic I/O error.
    Io(io::Error),
    /// A resampling filter produced results of dimensions
    /// other the ones specified by it's arguments.
    MismatchedDimensions(u32, (u32, u32)),
}

impl SourceImage {
    /// Attempts to create a `SourceImage` from a given path.
    ///
    /// # Return Value
    /// * Returns `Ok(src)` if the file indicated by the `path` argument could be
    ///   successfully parsed into an image.
    /// * Returns `Err(io::Error::from(io::ErrorKind::Other))` if the image allocation failed
    ///   or if the file was not able to be accessed.
    /// * Returns `Err(io::Error::from(io::ErrorKind::InvalidInput))` if the image format is not
    ///   supported by `icon_baker`.
    /// * Returns `Err(io::Error::from(io::ErrorKind::InvalidData))` otherwise.
    ///
    /// # Example
    /// ```rust
    /// let img = SourceImage::open("source.png")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        match image::open(&path) {
            Ok(img) => Ok(SourceImage::from(img)),
            Err(ImageError::InsufficientMemory) => Err(io::Error::from(io::ErrorKind::Other)),
            Err(ImageError::IoError(err)) => Err(err),
            Err(ImageError::UnsupportedError(_)) => match Tree::from_file(path, &usvg::Options::default()) {
                Ok(img) => Ok(SourceImage::from(img)),
                Err(usvg::Error::InvalidFileSuffix) => Err(io::Error::from(io::ErrorKind::InvalidInput)),
                Err(usvg::Error::FileOpenFailed) => Err(io::Error::from(io::ErrorKind::Other)),
                _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
            },
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }

    /// Returns the width of the original image in pixels.
    pub fn width(&self) -> f64 {
        match self {
            SourceImage::Raster(ras) => ras.width() as f64,
            SourceImage::Svg(svg) => svg.svg_node().view_box.rect.width(),
        }
    }

    /// Returns the height of the original image in pixels.
    pub fn height(&self) -> f64 {
        match self {
            SourceImage::Raster(ras) => ras.height() as f64,
            SourceImage::Svg(svg) => svg.svg_node().view_box.rect.height(),
        }
    }

    /// Returns the dimensions of the original image in pixels.
    pub fn dimensions(&self) -> (f64, f64) {
        (self.width(), self.height())
    }
}

impl From<Tree> for SourceImage {
    fn from(svg: Tree) -> Self {
        SourceImage::Svg(svg)
    }
}

impl From<DynamicImage> for SourceImage {
    fn from(bit: DynamicImage) -> Self {
        SourceImage::Raster(bit)
    }
}

impl<K: AsSize> Error<K> {
    /// Converts `self` to a `Error<T>` using `f`.
    pub fn map<T: AsSize, F: FnOnce(K) -> T>(self, f: F) -> Error<T> {
        match self {
            Error::AlreadyIncluded(e) => Error::AlreadyIncluded(f(e)),
            Error::Io(err) => Error::Io(err),
            Error::MismatchedDimensions(e, g) => Error::MismatchedDimensions(e, g),
        }
    }
}

impl<K: AsSize + Debug + Eq> Display for Error<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AlreadyIncluded(_) => write!(
                f,
                "the icon already contains an entry associated with this key"
            ),
            Error::Io(err) => write!(f, "{}", err),
            Error::MismatchedDimensions(s, (w, h)) => write!(
                f,
                "{0}: expected {1}x{1}, got {2}x{3}",
                INVALID_DIM_ERR, s, w, h
            ),
        }
    }
}

impl<K: AsSize + Debug> Debug for Error<K> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::AlreadyIncluded(e) => write!(f, "Error::AlreadyIncluded({:?})", e),
            Error::Io(err) => write!(f, "Error::Io({:?})", err),
            Error::MismatchedDimensions(e, g) => {
                write!(f, "Error::MismatchedDimensions({}, {:?})", e, g)
            }
        }
    }
}

impl<K: AsSize + Debug + Eq> error::Error for Error<K> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        if let Error::Io(ref err) = self {
            Some(err)
        } else {
            None
        }
    }
}

impl<K: AsSize> From<io::Error> for Error<K> {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl<K: AsSize> Into<io::Error> for Error<K> {
    fn into(self) -> io::Error {
        match self {
            Error::Io(err) => err,
            Error::MismatchedDimensions(_, _) => io::Error::from(io::ErrorKind::InvalidData),
            _ => io::Error::from(io::ErrorKind::InvalidInput),
        }
    }
}
