//! A simple solution for encoding common icon file formats,
//!  such as `.ico` and `.icns`. This crate is mostly a wrapper
//!  for other libraries, unifying existing APIs into a single,
//!  cohesive interface.
//! 
//! This crate serves as **[IconBaker CLI's]
//! (https://github.com/GarkGarcia/icon-baker)** internal
//!  library.
//! 
//! # Overview
//! 
//! An icon stores a collection of small images of different
//!  sizes. Individial images within the icon are binded to a
//!  source image, which is rescaled to fit a particular size
//!  using a resampling filter.
//! 
//! Resampling filters are represented by functions that take
//!  a source image and a size and return a rescaled raw RGBA
//!  buffer. This allows the user of this crate to provide
//!  their custom resampling filter. Common resampling filters
//!  are provided by the `resample` module.
//! 
//! # Examples
//! 
//! ## General Usage
//! ```rust
//! use icon_baker::*;
//!  
//! fn main() -> icon_baker::Result<()> {
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
//! ```rust
//! use icon_baker::*;
//! use std::{io, fs::File};
//!  
//! fn main() -> io::Result<()> {
//!     let icon = PngSequence::new();
//! 
//!     /* Process the icon */
//! 
//!     let file = File::create("ou.icns")?;
//!     icon.write(file)
//! }
//! ```
//! 
//! # Supported Image Formats
//! | Format | Supported?                                         | 
//! | ------ | -------------------------------------------------- | 
//! | `PNG`  | All supported color types                          | 
//! | `JPEG` | Baseline and progressive                           | 
//! | `GIF`  | Yes                                                | 
//! | `BMP`  | Yes                                                | 
//! | `ICO`  | Yes                                                | 
//! | `TIFF` | Baseline(no fax support), `LZW`, PackBits          | 
//! | `WEBP` | Lossy(Luma channel only)                           | 
//! | `PNM ` | `PBM`, `PGM`, `PPM`, standard `PAM`                |
//! | `SVG`  | Limited(flat filled shapes only)                   |

pub extern crate nsvg;

use std::{result, error, convert::From, path::Path, io::{self, Write}, fmt::{self, Display}};
pub use nsvg::{image::{self, DynamicImage, RgbaImage, GenericImage}, SvgImage};

pub use crate::ico::Ico;
pub use crate::icns::Icns;
pub use crate::png_sequence::PngSequence;

pub type Size = u32;
pub type Result<T> = result::Result<T, Error>;

#[cfg(test)]
mod test;
mod ico;
mod icns;
mod png_sequence;
pub mod resample;

/// A generinic representation of an icon encoder.
pub trait Icon {
    /// Creates a new icon.
    /// 
    /// # Example
    /// ```rust
    /// let icon = Ico::new();
    /// ```
    fn new() -> Self;

    /// Adds an individual entry to the icon.
    /// # Arguments
    /// * `filter` The resampling filter that will be used to re-scale `source`.
    /// * `source` A reference to the source image this entry will be based on.
    /// * `size` The target size of the entry in pixels.
    /// 
    /// # Example
    /// ```rust
    /// use icon_baker::*;
    ///  
    /// fn main() -> icon_baker::Result<()> {
    ///     let icon = Ico::new();
    /// 
    ///     match SourceImage::from_path("image.svg") {
    ///         Some(img) => icon.add_entry(resample::linear, &img, 32),
    ///         None      => Ok(())
    ///     }
    /// }
    /// ```
    fn add_entry<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()>;

    /// Adds a serie of entries to the icon.
    /// # Arguments
    /// * `filter` The resampling filter that will be used to re-scale `source`.
    /// * `source` A reference to the source image this entry will be based on.
    /// * `size` A conteiner for the target sizes of the entrie in pixels.
    /// 
    /// # Example
    /// ```rust
    /// use icon_baker::*;
    ///  
    /// fn main() -> icon_baker::Result<()> {
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
    fn add_entries<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>,I: IntoIterator<Item = Size>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        sizes: I
    ) -> Result<()>;

    /// Writes the contents of the icon to `w`.
    /// 
    /// # Example
    /// ```rust
    /// use icon_baker::*;
    /// use std::{io, fs::File};
    ///  
    /// fn main() -> io::Result<()> {
    ///     let icon = PngSequence::new();
    /// 
    ///     /* Process the icon */
    /// 
    ///     let file = File::create("out.icns")?;
    ///     icon.write(file)
    /// }
    /// ```
    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()>;
}

/// A representation of a source image.
pub enum SourceImage {
    /// A generic raster image.
    Bitmap(DynamicImage),
    /// A svg-encoded vector image.
    Svg(SvgImage)
}

#[derive(Debug)]
/// The error type operations of the Icon trait.
pub enum Error {
    Nsvg(nsvg::Error),
    Image(image::ImageError),
    Io(io::Error)
}

impl SourceImage {
    /// Attempts to create a `SourceImage` from a given path.
    /// 
    /// The `SourceImage::from<DynamicImage>` and `SourceImage::from<SvgImage>`
    /// methods should always be preferred.
    /// 
    /// # Example
    /// ```rust
    /// let img = SourceImage::from_path("source.png")?;
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        match image::open(&path) {
            Ok(bit) => Some(SourceImage::Bitmap(bit)),
            Err(_)  => match nsvg::parse_file(
                path.as_ref(),
                nsvg::Units::Pixel,
                96.0
            ) {
                Ok(svg) => Some(SourceImage::Svg(svg)),
                Err(_)  => None
            }
        }
    }

    /// Returns the width of the original image in pixels.
    pub fn width(&self) -> f32 {
        match self {
            SourceImage::Bitmap(bit) => bit.width() as f32,
            SourceImage::Svg(svg)    => svg.width()
        }
    }

    /// Returns the height of the original image in pixels.
    pub fn height(&self) -> f32 {
        match self {
            SourceImage::Bitmap(bit) => bit.height() as f32,
            SourceImage::Svg(svg)    => svg.height()
        }
    }

    /// Returns the dimentions of the original image in pixels.
    pub fn dimentions(&self) -> (f32, f32) {
        (self.width(), self.height())
    }
}

impl From<SvgImage> for SourceImage {
    fn from(svg: SvgImage) -> Self {
        SourceImage::Svg(svg)
    }
}

impl From<DynamicImage> for SourceImage {
    fn from(bit: DynamicImage) -> Self {
        SourceImage::Bitmap(bit)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Nsvg(err)  => write!(f, "{}", err),
            Error::Image(err) => write!(f, "{}", err),
            Error::Io(err)    => write!(f, "{}", err)
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Nsvg(err)  => err.description(),
            Error::Image(err) => err.description(),
            Error::Io(err)    => err.description()
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Nsvg(err)   => err.source(),
            Error::Image(err)  => err.source(),
            Error::Io(ref err) => Some(err)
        }
    }
}
