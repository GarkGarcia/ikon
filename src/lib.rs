//! A simple solution for encoding common icon file-formats, such as `.ico` and `.icns`. 
//! 
//! This crate is mostly a wrapper for other libraries, unifying existing APIs into a single, cohesive 
//! interface. It serves as **[IconPie's](https://github.com/GarkGarcia/icon-pie)** internal library.
//! 
//! # Overview
//! 
//! An _icon_ consists of a set of _entries_. An _entry_ is simply an image that has a particular size.
//! **IconBaker** simply automates the process of re-scaling pictures and combining them into an _icon_.
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
//! use icon_baker::{Ico, SourceImage, Icon};
//! use icon_baker::Error as IconError;
//!  
//! fn example() -> Result<(), IconError> {
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

pub use resvg::{usvg, raqote};
use std::{error, convert::From, path::{Path, PathBuf}, io::{self, Write}, fs::File, fmt::{self, Display, Debug}};
use image::{DynamicImage, GenericImageView};
use crate::usvg::Tree;

pub use crate::ico::Ico;
pub use crate::icns::Icns;
pub use crate::png_sequence::PngSequence;

#[cfg(test)]
mod test;
mod ico;
mod icns;
mod png_sequence;
pub mod resample;

const STD_CAPACITY: usize = 7;
const INVALID_DIM_ERR: &str = "a resampling filter return images of invalid resolution";

/// A generic representation of an icon encoder.
pub trait Icon<E: AsRef<u32> + Debug + Eq> {
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
    /// * `filter` The resampling filter that will be used to re-scale `source`.
    /// * `source` A reference to the source image this entry will be based on.
    /// * `size` The target size of the entry in pixels.
    /// 
    /// # Return Value
    /// * Returns `Err(Error::InvalidSize(_))` if the dimensions provided in the
    ///  `size` argument are not supported.
    /// * Returns `Err(Error::Image(ImageError::DimensionError))`
    ///  if the resampling filter provided in the `filter` argument produces
    ///  results of dimensions other than the ones specified by `size`.
    /// * Otherwise return `Ok(())`.
    /// 
    /// # Example
    /// ```rust
    /// use icon_baker::{Ico, SourceImage, Icon};
    /// use icon_baker::Error as IconError;
    ///  
    /// fn example() -> Result<(), IconError> {
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
        entry: E
    ) -> Result<(), Error<E>>;

    /// Adds a series of entries to the icon.
    /// # Arguments
    /// * `filter` The resampling filter that will be used to re-scale `source`.
    /// * `source` A reference to the source image this entry will be based on.
    /// * `size` A container for the target sizes of the entries in pixels.
    /// 
    /// # Return Value
    /// * Returns `Err(Error::InvalidSize(_))` if the dimensions provided in the
    ///  `size` argument are not supported.
    /// * Returns `Err(Error::Image(ImageError::DimensionError))`
    ///  if the resampling filter provided in the `filter` argument produces
    ///  results of dimensions other than the ones specified by `size`.
    /// * Otherwise return `Ok(())`.
    /// 
    /// # Example
    /// ```rust
    /// use icon_baker::{Icns, SourceImage, Icon};
    /// use icon_baker::Error as IconError;
    ///  
    /// fn example() -> Result<(), IconError> {
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
    fn add_entries<F: FnMut(&SourceImage, u32) -> DynamicImage, I: IntoIterator<Item = E>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        entries: I
    ) -> Result<(), Error<E>> {
        for entry in entries {
            self.add_entry(|src, size| filter(src, size), source, entry)?;
        }

        Ok(())
    }

    /// Writes the contents of the icon to `w`.
    /// 
    /// # Example
    /// ```rust
    /// use icon_baker::*;
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
    /// ```rust
    /// use icon_baker::*;
    /// use std::{io, fs::File};
    ///  
    /// fn example() -> io::Result<()> {
    ///     let icon = Ico::new();
    /// 
    ///     /* Process the icon */
    /// 
    ///     icon.save("./output/out.ico")
    /// }
    /// ```
    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<()> {
        let mut file = File::create(path.as_ref())?;
        self.write(&mut file)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entry(u32);

#[derive(Clone, Debug, Eq, Hash)]
pub struct NamedEntry(u32, PathBuf);

#[derive(Clone)]
/// A representation of a source image.
pub enum SourceImage {
    /// A generic raster image.
    Raster(DynamicImage),
    /// A svg-encoded vector image.
    Svg(Tree)
}

#[derive(Debug)]
/// The error type for operations of the `Icon` trait.
pub enum Error<E: AsRef<u32> + Debug + Eq> {
    /// The `Icon` instance already includes this entry.
    AlreadyIncluded(E),
    /// The return value of a resampling filter has invalid
    /// dimensions: the dimensions do not match the ones
    /// specified in the application of the filter.
    InvalidDimensions(u32, (u32, u32)),
    /// An unsupported size was suplied to an `Icon` operation.
    InvalidSize(u32),
    /// Generic I/O error.
    Io(io::Error)
}

impl AsRef<u32> for Entry {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl NamedEntry {
    /// Creates a `NamedEntry` from a reference to a `Path`.
    /// # Example
    /// ```rust
    /// let entry = NamedEntry::from(32, &"icons/32/icon.png");
    /// ```
    pub fn from<P: AsRef<Path>>(size: u32, path: &P) -> Self {
        NamedEntry(size, PathBuf::from(path.as_ref()))
    }
}

impl AsRef<u32> for NamedEntry {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl PartialEq for NamedEntry {
    fn eq(&self, other: &NamedEntry) -> bool {
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
            .ok().map(|svg| SourceImage::from(svg))
    }

    /// Returns the width of the original image in pixels.
    pub fn width(&self) -> f64 {
        match self {
            SourceImage::Raster(ras) => ras.width() as f64,
            SourceImage::Svg(svg)    => svg.svg_node().view_box.rect.width()
        }
    }

    /// Returns the height of the original image in pixels.
    pub fn height(&self) -> f64 {
        match self {
            SourceImage::Raster(ras) => ras.height() as f64,
            SourceImage::Svg(svg)    => svg.svg_node().view_box.rect.height()
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

impl<E: AsRef<u32> + Debug + Eq> Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AlreadyIncluded(_) => write!(f, "the icon already includes this entry"),
            Error::InvalidSize(s)     => write!(f, "{0}x{0} icons are not supported", s),
            Error::Io(err)            => write!(f, "{}", err),
            Error::InvalidDimensions(s, (w, h))
                => write!(f, "{0}: expected {1}x{1} and got {2}x{3}", INVALID_DIM_ERR, s, w, h)
        }
    }
}

impl<E: AsRef<u32> + Debug + Eq> error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        if let Error::Io(ref err) = self {
            Some(err)
        } else {
            None
        }
    }
}

impl<E: AsRef<u32> + Debug + Eq> From<io::Error> for Error<E> {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl<E: AsRef<u32> + Debug + Eq> Into<io::Error> for Error<E> {
    fn into(self) -> io::Error {
        match self {
            Error::Io(err)                 => err,
            Error::InvalidDimensions(_, _) => io::Error::from(io::ErrorKind::InvalidData),
            _                              => io::Error::from(io::ErrorKind::InvalidInput)
        }
    }
}
