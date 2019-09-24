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
//! ```rust
//! use icon_baker::{Ico, SourceImage, Icon, Error};
//!  
//! fn example() -> Result<(), Error> {
//!     let icon = Ico::new();
//! 
//!     match SourceImage::from_path("image.svg") {
//!         Some(img) => icon.add_entry(resample::linear, &img, 32),
//!         None      => Ok(())
//!     }
//! }
//! ```
//! 
//! ## Writing to a File
//! 
//! ```rust
//! use icon_baker::*;
//! use std::{io, fs::File};
//!  
//! fn example() -> io::Result<()> {
//!     let icon = PngSequence::new();
//! 
//!     /* Process the icon */
//! 
//!     let file = File::create("out.icns")?;
//!     icon.write(file)
//! }
//! ```

pub extern crate image;
pub extern crate resvg;

pub use resvg::{raqote, usvg};
use crate::usvg::Tree;
use image::{DynamicImage, GenericImageView};
use std::{
    convert::From,
    error,
    fmt::{self, Debug, Display, Formatter},
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub mod icns;
pub mod ico;
pub mod favicon;
pub mod png_sequence;
pub mod resample;
#[cfg(test)]
mod test;

const STD_CAPACITY: usize = 7;
const INVALID_DIM_ERR: &str =
    "a resampling filter returned an image of dimensions other than the ones specified by it's arguments";

/// A generic representation of an icon encoder.
pub trait Icon where Self::Key: AsRef<u32> {
    type Key;

    /// Creates a new icon.
    ///
    /// # Example
    /// ```rust
    /// let icon = Ico::new();
    /// ```
    fn new() -> Self;

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
    ///
    ///     match SourceImage::from_path("image.svg") {
    ///         Some(img) => icon.add_entry(resample::linear, &img, 32),
    ///         None      => Ok(())
    ///     }
    /// }
    /// ```
    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
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
    ///
    ///     match SourceImage::from_path("image.svg") {
    ///         Some(img) => icon.add_entries(
    ///             resample::linear,
    ///             &img,
    ///             vec![32, 64, 128]
    ///         ),
    ///         None => Ok(())
    ///     }
    /// }
    /// ```
    fn add_entries<F: FnMut(&SourceImage, u32) -> DynamicImage, I: IntoIterator<Item = Self::Key>>(
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
    /// use icon_baker::png_sequence::PngSequence;
    /// use std::{io, fs::File};
    ///  
    /// fn example() -> io::Result<()> {
    ///     let icon = PngSequence::new();
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// A _key type_ for simple icons that only associate images
/// with their dimensions. Usefull for icon formats such as the
/// `.ico` and `.icns` file formats.
pub struct SizeKey(u32);

#[derive(Clone, Debug, Eq, Hash)]
/// An _key type_ for _icon formats_ that consist of a
/// collectio of files labeled by _path_, such as _png sequences_ or
/// _[FreeDesktop icon themes](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html)_.
pub struct PathKey(u32, PathBuf);

#[derive(Clone)]
/// A uniun type for raster and vector graphics.
pub enum SourceImage {
    /// A generic raster image.
    Raster(DynamicImage),
    /// A svg-encoded vector image.
    Svg(Tree),
}

/// The error type for operations of the `Icon` trait.
pub enum Error<K: AsRef<u32>> {
    /// The `Icon` instance already includes an entry associated with this key.
    AlreadyIncluded(K),
    /// Generic I/O error.
    Io(io::Error),
    /// Unsupported dimensions were suplied to an `Icon`
    /// operation.
    InvalidDimensions(u32),
    /// A resampling filter produced results of dimensions
    /// other the ones specified by it's arguments.
    MismatchedDimensions(u32, (u32, u32)),
}

impl AsRef<u32> for SizeKey {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl PathKey {
    /// Creates a `NamedEntry` from a reference to a `Path`.
    /// # Example
    /// ```rust
    /// let key = NamedEntry::from(32, &"icons/32/icon.png");
    /// ```
    pub fn from<P: AsRef<Path>>(size: u32, path: &P) -> Self {
        PathKey(size, PathBuf::from(path.as_ref()))
    }
}

impl AsRef<u32> for PathKey {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl PartialEq for PathKey {
    fn eq(&self, other: &PathKey) -> bool {
        self.1 == other.1
    }
}

impl SourceImage {
    /// Attempts to create a `SourceImage` from a given path.
    ///
    /// The `SourceImage::from::<image::DynamicImage>` and `SourceImage::from::<usvg::Tree>`
    /// methods should always be preferred.
    ///
    /// # Return Value
    /// * Returns `Some(src)` if the file indicated by the `path` argument could be
    ///   successfully parsed into an image.
    /// * Returns `None` otherwise.
    ///
    /// # Example
    /// ```rust
    /// let img = SourceImage::open("source.png")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Option<Self> {
        if let Ok(ras) = image::open(&path) {
            return Some(SourceImage::from(ras));
        }

        Tree::from_file(&path, &usvg::Options::default())
            .ok()
            .map(|svg| SourceImage::from(svg))
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

impl<K: AsRef<u32>> Error<K> {
    /// Converts `self` to a `Error<T>` using `f`.
    pub fn map<T: AsRef<u32>, F: FnOnce(K) -> T>(
        self,
        f: F
    ) -> Error<T> {
        match self {
            Error::AlreadyIncluded(e) => Error::AlreadyIncluded(f(e)),
            Error::InvalidDimensions(size) => Error::InvalidDimensions(size),
            Error::Io(err) => Error::Io(err),
            Error::MismatchedDimensions(e, g) => Error::MismatchedDimensions(e, g),
        }
    }
}

impl<K: AsRef<u32> + Debug + Eq> Display for Error<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AlreadyIncluded(_) => write!(f, "the icon already contains an entry associated with this key"),
            Error::InvalidDimensions(s) => write!(f, "{0}x{0} icons are not supported", s),
            Error::Io(err) => write!(f, "{}", err),
            Error::MismatchedDimensions(s, (w, h)) => write!(
                f,
                "{0}: expected {1}x{1}, got {2}x{3}",
                INVALID_DIM_ERR, s, w, h
            ),
        }
    }
}

impl <K: AsRef<u32> + Debug> Debug for Error<K> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::AlreadyIncluded(e) => write!(f, "Error::AlreadyIncluded({:?})", e),
            Error::InvalidDimensions(s) => write!(f, "Error::InvalidDimensions({})", s),
            Error::Io(err) => write!(f, "Error::Io({:?})", err),
            Error::MismatchedDimensions(e, g) => write!(f, "Error::MismatchedDimensions({}, {:?})", e, g)
        }
    }
}

impl<K: AsRef<u32> + Debug + Eq> error::Error for Error<K> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        if let Error::Io(ref err) = self {
            Some(err)
        } else {
            None
        }
    }
}

impl<K: AsRef<u32>> From<io::Error> for Error<K> {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl<K: AsRef<u32>> Into<io::Error> for Error<K> {
    fn into(self) -> io::Error {
        match self {
            Error::Io(err) => err,
            Error::MismatchedDimensions(_, _) => io::Error::from(io::ErrorKind::InvalidData),
            _ => io::Error::from(io::ErrorKind::InvalidInput),
        }
    }
}
