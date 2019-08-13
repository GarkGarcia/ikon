//! A simple solution for generating `.ico` and `.icns` icons. This crate serves as **IconBaker CLI's** internal library.
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

extern crate tar;
extern crate png_encode_mini;
extern crate ico;
extern crate icns;
pub extern crate nsvg;

use std::{convert::From, path::Path, marker::Sized, io::{self, Write}, collections::HashMap};
use nsvg::{image::{DynamicImage, RgbaImage, GenericImage}, SvgImage};
pub use nsvg::image;

const MAX_ICO_SIZE: u32 = 265;
const VALID_ICNS_SIZES: [u32;7] = [16, 32, 64, 128, 256, 512, 1024];

// A representation of an icon's size in pixel units.
pub type Size = u32;
pub type Result<T> = std::result::Result<T, Error>;
type SourceMap<'a> = HashMap<Size, &'a SourceImage>;

mod write;
pub mod resample;
pub mod prelude {
    pub use super::{Icon, SourceImage, FromPath};
}

/// A generic representation of an icon.
pub struct Icon<W: Write> {
    raw: RawIcon<W>
}

struct RawIco<W: Write> {
    icon_dir: ico::IconDir,
    writer: W
}

struct RawIcns<W: Write> {
    icon_family: icns::IconFamily,
    buf_writer: io::BufWriter<W>
}

struct RawPngSequence<W: Write> {
    tar_builder: tar::Builder<W>
}

enum RawIcon<W: Write> {
    Ico(RawIco<W>),
    Icns(RawIcns<W>),
    PngSequence(RawPngSequence<W>)
}

trait GenericIcon {
    fn add_icon<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()>;
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
    Io(io::Error),
    InvalidIcoSize(Size),
    InvalidIcnsSize(Size),
    SizeAlreadyIncluded(Size)
}

/// Trait for constructing structs from a given path.
pub trait FromPath where Self: Sized {
    /// Constructs `Self` from a given path.
    fn from_path<P: AsRef<Path>>(path: P) -> Option<Self>;
}

impl<W: Write> RawIco<W> {
    pub fn new(w: W) -> Self {
        RawIco { icon_dir: ico::IconDir::new(ico::ResourceType::Icon), writer: w }
    }
}

impl<W: Write> GenericIcon for RawIco<W> {
    fn add_icon<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()> {
            let icon = filter(source, size)?;
            let size = icon.width();
            let data = ico::IconImage::from_rgba_data(size, size, icon.clone().into_vec());

            match ico::IconDirEntry::encode(&data) {
                Ok(entry) => self.icon_dir.add_entry(entry),
                Err(err) => return Err(Error::Io(err))
            }

            let mut buf: Vec<u8> = Vec::new();

            match self.icon_dir.write::<&mut [u8]>(buf.as_mut()) {
                Ok(_) => match self.writer.write_all(buf.as_mut()) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(Error::Io(err))
                },
                Err(err) => Err(Error::Io(err))
            }
    }
}

impl <W: Write> RawIcns<W> {
    pub fn new(w: W) -> Self {
        RawIcns { icon_family: icns::IconFamily::new(), buf_writer: io::BufWriter::new(w) }
    }
}

impl<W: Write> GenericIcon for RawIcns<W> {
    fn add_icon<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()> {
        let icon = filter(source, size)?;

        match icns::Image::from_data(icns::PixelFormat::RGBA, size, size, icon.into_vec()) {
            Ok(icon) => if let Err(err) = self.icon_family.add_icon(&icon) {
                return Err(Error::Io(err))
            },
            Err(err) => return Err(Error::Io(err))
        }

        let mut buf: Vec<u8> = Vec::new();

        match self.icon_family.write::<&mut [u8]>(buf.as_mut()) {
            Ok(_) => match self.buf_writer.write_all(buf.as_mut()) {
                Ok(_) => Ok(()),
                Err(err) => Err(Error::Io(err))
            },
            Err(err) => Err(Error::Io(err))
        }
    }
}

impl<W: Write> RawPngSequence<W> {
    pub fn new(w: W) -> Self {
        RawPngSequence { tar_builder: tar::Builder::new(w) }
    }
}

impl<W: Write> GenericIcon for RawPngSequence<W> {
    fn add_icon<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()> {
        let icon = filter(source, size)?;
        let size = icon.width();
    
        // Encode the pixel data as PNG and store it in a Vec<u8>
        let mut data = Vec::with_capacity(icon.len());
        if let Err(err) = png_encode_mini::write_rgba_from_u8(&mut data, &icon.into_raw(), size, size) {
            return Err(Error::Io(err));
        }
    
        let file_name = format!("/{}.png", size);
    
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_cksum();
    
        if let Err(err) = self.tar_builder.append_data::<String, &[u8]>(&mut header, file_name, data.as_ref()) {
            Err(Error::Io(err))
        } else {
            Ok(())
        }
    }
}

impl<W: Write> Icon<W> {

    /// Creates an `Icon` with the `Ico` icon type.
    /// # Arguments
    /// * `capacity` The target capacity for the underliyng `HashMap<IconOptions, &SourceImage>`.
    /// 
    /// It is important to note that although the returned `Icon` has the capacity specified, the `Icon` will have zero sizes.
    /// For an explanation of the difference between length and capacity, see
    /// [*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
    pub fn ico(w: W) -> Self {
        Icon { raw: RawIcon::Ico(RawIco::new(w)) }
    }

    /// Creates an `Icon` with the `Icns` icon type.
    /// # Arguments
    /// * `capacity` The target capacity for the underliyng `HashMap<IconOptions, &SourceImage>`.
    /// 
    /// It is important to note that although the returned `Icon` has the capacity specified, the `Icon` will have zero sizes.
    /// For an explanation of the difference between length and capacity, see
    /// [*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
    pub fn icns(w: W) -> Self {
        Icon { raw: RawIcon::Icns(RawIcns::new(w)) }

    }

    /// Creates an `Icon` with the `PngSequece` icon type.
    /// # Arguments
    /// * `capacity` The target capacity for the underliyng `HashMap<IconOptions, &SourceImage>`.
    /// 
    /// It is important to note that although the returned `Icon` has the capacity specified, the `Icon` will have zero sizes.
    /// For an explanation of the difference between length and capacity, see
    /// [*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
    pub fn png_sequence(w: W) -> Self {
        Icon { raw: RawIcon::PngSequence(RawPngSequence::new(w)) }
    }

    /// Adds a size binding to the icon.
    /// 
    /// Returns `Err(_)` if the specified size is invalid or is already included in the Icon.
    /// Returns `Ok(())` otherwise.
    pub fn add_icon<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()> {

        match &mut self.raw {
            RawIcon::Ico(mut raw)         => raw.add_icon(filter, source, size),
            RawIcon::Icns(mut raw)        => raw.add_icon(filter, source, size),
            RawIcon::PngSequence(mut raw) => raw.add_icon(filter, source, size)
        }
    }

    /// Adds a series sizes binding to the icon.
    /// 
    /// Returns `Err(_)` if any of the specified sizes is invalid or is already included in the Icon.
    /// Returns `Ok(())` otherwise.
    pub fn add_icons<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>, I: ExactSizeIterator<Item = Size>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        sizes: I
    ) -> Result<()> {

        for size in sizes {
            if let Err(err) = self.add_icon(filter, source, size) {
                return Err(err);
            }
        }

        Ok(())
    }
}

impl SourceImage {
    /// Returns the width of the original image in pixels.
    pub fn width(&self) -> f32 {
        match self {
            SourceImage::Bitmap(bit) => bit.width() as f32,
            SourceImage::Svg(svg) => svg.width()
        }
    }

    /// Returns the height of the original image in pixels.
    pub fn height(&self) -> f32 {
        match self {
            SourceImage::Bitmap(bit) => bit.height() as f32,
            SourceImage::Svg(svg) => svg.height()
        }
    }

    /// Returns the dimentions of the original image in pixels.
    pub fn dimentions(&self) -> (f32, f32) {
        match self {
            SourceImage::Bitmap(bit) => (bit.width() as f32, bit.height() as f32),
            SourceImage::Svg(svg) => (svg.width(), svg.height())
        }
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
        } else if let Ok(svg) = nsvg::parse_file(path.as_ref(), nsvg::Units::Pixel, 96.0) {
            Some(SourceImage::Svg(svg))
        } else {
            None
        }
    }
}

/* #[cfg(test)]
mod test {
    use crate::{Icon, SourceImage, FromPath};
    use std::fs::File;

    #[test]
    fn test_write() {
        let mut icon = Icon::ico(2);
        let img1 = SourceImage::from_path("test1.svg").unwrap();
        let img2 = SourceImage::from_path("test2.svg").unwrap();

        let _ = icon.add_size(32, &img1);
        let _ = icon.add_size(64, &img2);

        let file = File::create("test.ico").unwrap();

        let _ = icon.write(file, crate::resample::linear);
    }
} */