//! A simple solution for generating `.ico` and `.icns` icons. This crate serves as **IconBaker CLI's** internal library.
//! 
//! # Usage
//! ```rust
//! use icon_baker::prelude::*;
//! 
//! const N_ENTRIES: usize = 1;
//! 
//! fn main() {
//!     // Creating the icon
//!     let mut icon = Icon::ico(N_ENTRIES);
//! 
//!     // Importing the source image
//!     let src_image = SourceImage::from_path("img.jpg").unwrap();
//! 
//!     // Adding the sizes
//!     icon.add_sizes(&vec![32, 64], &src_image).unwrap();
//! }
//! ```
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

use std::{convert::From, path::Path, marker::Sized, io::{self, Write}};
pub use nsvg::{image::{self, DynamicImage, RgbaImage, GenericImage}, SvgImage};

pub use crate::ico::Ico;
pub use crate::icns::Icns;
pub use png_sequence::PngSequence;

// A representation of an icon's size in pixel units.
pub type Size = u32;
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test;
mod ico;
mod icns;
mod png_sequence;
pub mod resample;
pub mod prelude {
    pub use crate::{Icon, Ico, Icns, PngSequence, SourceImage, FromPath, resample};
}

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
    /// use icon_baker::prelude::*;
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
    /// use icon_baker::prelude::*;
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
    /// use icon_baker::prelude::*;
    /// use std::{io, fs::File};
    ///  
    /// fn main() -> io::Result<()> {
    ///     let icon = PngSequence::new();
    /// 
    ///     /* Process the icon */
    /// 
    ///     let file = File::create("icon.ico")?;
    ///     icon.write(file)
    /// }
    /// ```
    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()>;
}

/// A trait for constructing structs from a given path.
pub trait FromPath where Self: Sized {
    /// Constructs `Self` from a given path.
    fn from_path<P: AsRef<Path>>(path: P) -> Option<Self>;
}

/// A representation of a bitmap or an svg image.
pub enum SourceImage {
    Bitmap(DynamicImage),
    Svg(SvgImage)
}

#[derive(Debug)]
pub enum Error {
    Nsvg(nsvg::Error),
    Image(image::ImageError),
    Io(io::Error)
}

impl SourceImage {
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
    fn from(din: DynamicImage) -> Self {
        SourceImage::Bitmap(din)
    }
}

impl FromPath for SourceImage {
    fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        if let Ok(din) = image::open(&path) {
            Some(SourceImage::Bitmap(din))
        } else if let Ok(svg) = nsvg::parse_file(
            path.as_ref(),
            nsvg::Units::Pixel,
            96.0
        ) {
            Some(SourceImage::Svg(svg))
        } else {
            None
        }
    }
}
