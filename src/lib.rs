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
//! _[path](https://en.wikipedia.org/wiki/Path_%28computing%29)_, such as
//! _[FreeDesktop icon themes](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html)_
//! , are labeled by _path_.
//! 
//! Note that, since the dimensions of the _images_ contained in an _entry_ are dictated by their
//! associated _entries_, every _key_ must be convertible to a _positive integers_. Therefore, all
//! _key types_ are required to implement `AsSize`.
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
//! The `IconEncoder::add_entry` can be used to automatically resample
//! _source images_ and converts them to _entries_ in an icon.
//! 
//! ```rust
//! use icon_baker::{ico::{Ico, Key}, Image, IconEncoder, EncodingError};
//! 
//! fn example() -> Result<(), EncodingError> {
//!     let icon = Ico::new();
//!     let src = Image::open("image.svg")?;
//! 
//!     icon.add_entry(resample::linear, &img, Key(32))
//! }
//! ```
//! 
//! ## Writing to Disk
//! 
//! Implementors of the `IconEncoder` trait can be written to any object
//! that implements `io::Write` with the `IconEncoder::write` method.
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
//! disk with `IconEncoder::save` method.
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
pub use crate::error::{EncodingError, ResampleError};
use image::{DynamicImage, GenericImageView, ImageError, ImageFormat};
pub use resvg::{raqote, usvg};
use std::{
    convert::From,
    fs::File,
    io::{self, Read, BufReader},
    path::Path,
};
pub use crate::encode::{Encoder, Write, Save};

pub mod resample;
pub mod encode;
mod error;
#[cfg(test)]
mod test;

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
        let mut file = File::open(path)?;

        // Read the file's signature
        let mut signature: [u8;8] = [0;8];
        file.read_exact(&mut signature)?;

        match signature {
            [0x89, 0x50, 0x4e, 0x47, 0xd, 0xa, 0x1a, 0xa] => load_raster(file, ImageFormat::PNG),
            [0xff, 0xd8, 0xff, _, _, _, _, _] => load_raster(file, ImageFormat::JPEG),
            [0x47, 0x49, 0x46, 0x38, 0x37, 0x61, _, _]
            | [0x47, 0x49, 0x46, 0x38, 0x39, 0x61, _, _] => load_raster(file, ImageFormat::GIF),
            [0x42, 0x4d, _, _, _, _, _, _] => load_raster(file, ImageFormat::BMP),
            [0x57, 0x45, 0x42, 0x50, _, _, _, _] => load_raster(file, ImageFormat::WEBP),
            _ => load_vector(file)
        }
    }

    /// Rasterizes the `Image` to a `DynamicImage`.
    /// 
    /// For _raster graphics_ the moethod simply applies the resampling filter
    /// specified by the `filter` argument. For _vector graphics_, the method
    /// rasterizes the image to fit the dimensions specified `size` using
    /// [_linear interpolation_](https://en.wikipedia.org/wiki/Linear_interpolation)
    /// and [_anti-aliasing_](https://en.wikipedia.org/wiki/Anti-aliasing).
    /// 
    /// # Example
    /// 
    /// ```rust
    /// if let Ok(raster) = image.rasterize(resample::linear, 32) {
    ///     // Process raster...
    /// }
    /// ```
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

    /// Returns the width of the image in pixel units.
    pub fn width(&self) -> f64 {
        match self {
            Image::Raster(ras) => ras.width() as f64,
            Image::Svg(svg) => svg.svg_node().view_box.rect.width(),
        }
    }

    /// Returns the height of the image in pixel units.
    pub fn height(&self) -> f64 {
        match self {
            Image::Raster(ras) => ras.height() as f64,
            Image::Svg(svg) => svg.svg_node().view_box.rect.height(),
        }
    }

    /// Returns the dimensions of the image in pixel units.
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

/// Loads raster graphics to an `Image`.
fn load_raster(file: File, format: ImageFormat) -> io::Result<Image> {
    match image::load(BufReader::new(file), format) {
        Ok(img) => Ok(Image::from(img)),
        Err(ImageError::InsufficientMemory) => Err(io::Error::from(io::ErrorKind::Other)),
        Err(ImageError::IoError(err)) => Err(err),
        _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
    }
}

/// Loads vector graphics to an `Image`.
fn load_vector(mut file: File) -> io::Result<Image> {
    file.sync_all()?;
    let len = file.metadata()?.len() as usize;

    let mut contents = Vec::with_capacity(len);
    file.read_to_end(&mut contents)?;

    match Tree::from_data(contents.as_ref(), &usvg::Options::default()) {
        Ok(img) => Ok(Image::from(img)),
        Err(usvg::Error::InvalidFileSuffix) => {
            Err(io::Error::from(io::ErrorKind::InvalidInput))
        }
        Err(usvg::Error::FileOpenFailed) => Err(io::Error::from(io::ErrorKind::Other)),
        _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
    }
}
