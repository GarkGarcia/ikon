//! A robust, flexible framework for creating encoders and decoders for various _icon formats_.
//! 
//! # Overview
//! 
//! **Ikon** is intended to be used as a framework for developers interested in creating encoders and decoders
//! for _various icon formats_ such as `.ico` files and _favicon_ schemes. It **does not** come with any encoders
//! or decoders out of the box.
//! 
//! Instead, it simply automates much of the hard work of _encoding_, _decoding_ and _resampling_ different
//! _image formats_, as well as provides powerfull abstractions, allowing developers to concentrate on the more
//! relevant problems.
//! 
//! _Icons_ are represented as maps between _keys_ and _images_. An _entry_ is a _key-value_ pair contained
//! in an _icon_. The type of the _keys_ of an _icon_ is what determines how it can be indexed. 
//! 
//! ## Keys
//! 
//! Each _icon format_ is associated with a particular type of _key_. The type of the _keys_ of an _icon_ is
//! what determines how it can be indexed. Each _key_ can only be associated with a single _image_.
//! 
//! Since the _keys_ of an _icon_ also encode information about the dimensions of it's associated _image_,
//! `Encode::Key` and `Decode::Key` are required to implement `AsSize`.
//! 
//! ## Resampling
//! 
//! Raster graphics are scaled using resampling filters, which are represented by _functions that take a_
//! _source image and a size and return a re-scaled image_.
//! 
//! This allows the users of `ikon` and any of it's dependant crates to provide their custom resampling
//! filters. Common resampling filters are provided in the
//! [`resample`](https://docs.rs/ikon/0.1.0-beta.3/ikon/resample/index.html) module. The `resample`
//! module also exposes the `resample::apply` function, which applies a resampling filter to an _image_
//! and checks if the outputted result matches the dimensions specified by the filter's arguments.

pub extern crate image;
pub extern crate resvg;

use crate::{usvg::Tree, resample::ResampleError};
use image::{DynamicImage, GenericImageView, ImageError, ImageFormat};
pub use resvg::{raqote, usvg};
use std::{
    convert::From,
    fs::File,
    io::{self, Read, Seek, BufReader, SeekFrom},
    path::Path,
};

pub mod resample;
pub mod encode;
pub mod decode;
#[cfg(test)]
mod test;

/// A trait for types that represent the dimesions of an icon.
pub trait AsSize {
    fn as_size(&self) -> (u32, u32);
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
    #[inline]
    /// Attempts to create a `Image` from a given path.
    ///
    /// # Return Value
    /// 
    /// * Returns `Ok(src)` if the file indicated by the `path` argument could be
    ///   successfully parsed into an image.
    /// * Returns `Err(io::Error::from(io::ErrorKind::Other))` if the image allocation failed
    ///   or if the file was not able to be accessed.
    /// * Returns `Err(io::Error::from(io::ErrorKind::InvalidInput))` if the image format is not
    ///   supported by `ikon`.
    /// * Returns `Err(io::Error::from(io::ErrorKind::InvalidData))` otherwise.
    ///
    /// # Example
    /// ```rust
    /// let img = Image::open("source.png")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        Self::load(File::open(path)?)
    }

    /// Attempts to create a `Image` from a byte stream.
    ///
    /// # Return Value
    /// 
    /// * Returns `Ok(src)` if the stram indicated by the `read` argument could be
    ///   successfully parsed into an image.
    /// * Returns `Err(io::Error::from(io::ErrorKind::Other))` if the image allocation failed.
    /// * Returns `Err(io::Error::from(io::ErrorKind::InvalidInput))` if the image format is not
    ///   supported by `ikon`.
    /// * Returns `Err(io::Error::from(io::ErrorKind::InvalidData))` otherwise.
    ///
    /// # Example
    /// ```rust
    /// let file = File::open("source.png")?;
    /// let img = Image::load(file)?;
    /// ```
    pub fn load<R: Read + Seek>(mut read: R) -> Result<Self, io::Error> {
        // Read the file's signature
        let mut signature: [u8;8] = [0;8];
        read.read_exact(&mut signature)?;
        read.seek(SeekFrom::Start(0))?;

        match signature {
            [0x89, 0x50, 0x4e, 0x47, 0xd, 0xa, 0x1a, 0xa] => load_raster(read, ImageFormat::PNG).map(Image::from),
            [0xff, 0xd8, 0xff, _, _, _, _, _] => load_raster(read, ImageFormat::JPEG).map(Image::from),
            [0x47, 0x49, 0x46, 0x38, 0x37, 0x61, _, _]
            | [0x47, 0x49, 0x46, 0x38, 0x39, 0x61, _, _] => load_raster(read, ImageFormat::GIF).map(Image::from),
            [0x42, 0x4d, _, _, _, _, _, _] => load_raster(read, ImageFormat::BMP).map(Image::from),
            [0x52, 0x49, 0x46, 0x46, _, _, _, _] => load_raster(read, ImageFormat::WEBP).map(Image::from),
            _ => load_vector(read).map(Image::from)
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
    pub fn rasterize<F: FnMut(&DynamicImage, (u32, u32)) -> io::Result<DynamicImage>>(
        &self,
        filter: F,
        size: (u32, u32),
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

impl AsSize for (u32, u32) {
    fn as_size(&self) -> (u32, u32) {
        *self
    }
}

impl AsSize for (u16, u16) {
    fn as_size(&self) -> (u32, u32) {
        (self.0 as u32, self.1 as u32)
    }
}

impl AsSize for (u8, u8) {
    fn as_size(&self) -> (u32, u32) {
        (self.0 as u32, self.1 as u32)
    }
}

/// Loads raster graphics to an `Image`.
fn load_raster<R: Read + Seek>(read: R, format: ImageFormat) -> io::Result<DynamicImage> {
    match image::load(BufReader::new(read), format) {
        Ok(img) => Ok(img),
        Err(ImageError::InsufficientMemory) => Err(io::Error::from(io::ErrorKind::Other)),
        Err(ImageError::IoError(err)) => Err(err),
        _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
    }
}

/// Loads vector graphics to an `Image`.
fn load_vector<R: Read + Seek>(mut read: R) -> io::Result<Tree> {
    // Combute the length of the file and return to the start of
    // the stream.
    let len = read.seek(SeekFrom::End(0))?;
    read.seek(SeekFrom::Start(0))?;

    let mut contents = Vec::with_capacity(len as usize);
    read.read_to_end(&mut contents)?;

    match Tree::from_data(contents.as_ref(), &usvg::Options::default()) {
        Ok(img) => Ok(img),
        Err(usvg::Error::InvalidFileSuffix) => {
            Err(io::Error::from(io::ErrorKind::InvalidInput))
        }
        Err(usvg::Error::FileOpenFailed) => Err(io::Error::from(io::ErrorKind::Other)),
        _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
    }
}
