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
//! use icon_baker::{ico::{Ico, Key}, Image, Icon, Error};
//!   
//! fn example() -> Result<(), Error> {
//!     let icon = Ico::new();
//!     let src = Image::open("image.svg")?;
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
pub use resvg::{
    raqote,
    usvg::{self, XmlIndent, XmlOptions},
};
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
pub mod encode;
#[cfg(test)]
mod test;

const STD_CAPACITY: usize = 7;
const MISMATCHED_DIM_ERR: &str =
    "a resampling filter returned an image of dimensions other than the ones specified by it's arguments";

/// A generic representation of an icon encoder.
pub trait Icon
where
    Self: Sized,
{
    type Key: AsSize + Send + Sync;

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
    /// * Returns `Err(IconError::AlreadyIncluded(_))` if the icon already contains
    ///   an entry associated with `key`.
    /// * Returns `Err(IconError::Resample(_))` if the resampling filter provided in
    ///   the `filter` argument fails produces results of dimensions other than the
    ///   ones specified by `key`.
    /// * Otherwise returns `Ok(())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use icon_baker::{Ico, Image, Icon, IconError};
    ///  
    /// fn example() -> Result<(), IconError> {
    ///     let icon = Ico::new();
    ///     let src = Image::open("image.svg")?;
    ///
    ///     icon.add_entry(resample::linear, &img, 32)
    /// }
    /// ```
    fn add_entry<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &Image,
        key: Self::Key,
    ) -> Result<(), IconError<Self::Key>>;

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
    /// * Returns `Err(IconError::AlreadyIncluded(_))` if the icon already contains an
    ///   entry associated with any of the items of `keys`.
    /// * Returns `Err(IconError::Resample(_))` if the resampling filter provided in
    ///   the `filter` argument fails or produces results of dimensions other than the
    ///   ones specified by the items of `keys`.
    /// * Otherwise returns `Ok(())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use icon_baker::{Icns, Image, Icon, IconError};
    ///  
    /// fn example() -> Result<(), IconError> {
    ///     let icon = Icns::new();
    ///     let src = Image::open("image.svg")?;
    ///
    ///     icon.add_entries(
    ///         resample::linear,
    ///         &src,
    ///         vec![32, 64, 128]
    ///     )
    /// }
    /// ```
    fn add_entries<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>, I: IntoIterator<Item = Self::Key>>(
        &mut self,
        mut filter: F,
        source: &Image,
        keys: I,
    ) -> Result<(), IconError<Self::Key>> {
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
pub enum Image {
    /// A generic raster image.
    Raster(DynamicImage),
    /// A svg-encoded vector image.
    Svg(Tree),
}

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

impl Image {
    /// Attempts to create a `Image` from a given path.
    ///
    /// # Return Value
    /// 
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
    /// let img = Image::open("source.png")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        match image::open(&path) {
            Ok(img) => Ok(Image::from(img)),
            Err(ImageError::InsufficientMemory) => Err(io::Error::from(io::ErrorKind::Other)),
            Err(ImageError::IoError(err)) => Err(err),
            Err(ImageError::UnsupportedError(_)) => {
                match Tree::from_file(path, &usvg::Options::default()) {
                    Ok(img) => Ok(Image::from(img)),
                    Err(usvg::Error::InvalidFileSuffix) => {
                        Err(io::Error::from(io::ErrorKind::InvalidInput))
                    }
                    Err(usvg::Error::FileOpenFailed) => Err(io::Error::from(io::ErrorKind::Other)),
                    _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
                }
            }
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }

    /// Rasterizes the `Image` to a `DynamicImage`.
    /// 
    /// For _raster graphics_ the moethod simply applies the resampling filter
    /// specified by the `filter` argument. For _vector graphics_, the method
    /// rasterizes the image to fit the dimensions specified `size` using
    /// linear interpolation and antialiasing.
    pub fn rasterize<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>>(
        &self,
        filter: F,
        size: u32,
    ) -> Result<DynamicImage, ResampleError> {
        match self {
            Self::Raster(ras) => resample::apply(filter, ras, size),
            Self::Svg(svg) => resample::svg(svg, size),
        }
    }

    /// Returns the width of the image in pixels.
    pub fn width(&self) -> f64 {
        match self {
            Image::Raster(ras) => ras.width() as f64,
            Image::Svg(svg) => svg.svg_node().view_box.rect.width(),
        }
    }

    /// Returns the height of the image in pixels.
    pub fn height(&self) -> f64 {
        match self {
            Image::Raster(ras) => ras.height() as f64,
            Image::Svg(svg) => svg.svg_node().view_box.rect.height(),
        }
    }

    /// Returns the dimensions of the image in pixels.
    pub fn dimensions(&self) -> (f64, f64) {
        (self.width(), self.height())
    }
}

impl From<Tree> for Image {
    fn from(svg: Tree) -> Self {
        Image::Svg(svg)
    }
}

impl From<DynamicImage> for Image {
    fn from(bit: DynamicImage) -> Self {
        Image::Raster(bit)
    }
}

unsafe impl Send for Image {}
unsafe impl Sync for Image {}

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

impl<K: AsSize + Send + Sync + Debug> error::Error for IconError<K> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
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

impl error::Error for ResampleError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
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
