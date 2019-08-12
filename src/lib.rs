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
//!     // Configuring the entry
//!     let entry = vec![(32, 32), (64, 64)]; // 32x32 and 64x64 sizes
//! 
//!     // Adding the entry
//!     icon.add_entry(entry, &src_image).unwrap();
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

extern crate zip;
extern crate png_encode_mini;
extern crate ico;
extern crate icns;
pub extern crate nsvg;

use std::{convert::From, path::Path, marker::Sized, io::{self, Write, Seek}, collections::{HashMap, BTreeSet}};
use nsvg::{image::{DynamicImage, RgbaImage, GenericImage}, SvgImage};
use zip::result::ZipError;
pub use nsvg::image;

const MAX_ICO_SIZE: u16 = 265;
const VALID_ICNS_SIZES: [(u16, u16);7] = [(16, 16), (32, 32), (64, 64), (128, 128), (256, 256), (512, 512), (1024, 1024)];

pub type Size = (u16, u16);
pub type Result<T> = std::result::Result<T, Error>;
type SourceMap<'a> = HashMap<Entry, &'a SourceImage>;

mod write;
pub mod resample;
pub mod prelude {
    pub use super::{Icon, Entry, IconType, SourceImage, FromPath};
}

#[macro_export]
macro_rules! entry {
    ($($x:expr), *) => ({
        let mut output = std::collections::BTreeSet::new();

        for &elem in &[$($x), *] {
            output.insert(elem);
        }

        output
    });
}

/// A generic representation of an icon.
pub struct Icon<'a> {
    source_map: SourceMap<'a>,
    icon_type: IconType
}

/// A representation of an entry's sizes.
pub type Entry = BTreeSet<Size>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconType {
    Ico,
    Icns,
    PngSequence
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
    Zip(ZipError),
    Io(io::Error),
    SizeAlreadyIncluded(Size),
    InvalidIcoSize(Size),
    InvalidIcnsSize(Size)
}

/// Trait for constructing structs from a given path.
pub trait FromPath where Self: Sized {
    /// Constructs `Self` from a given path.
    fn from_path<P: AsRef<Path>>(path: P) -> Option<Self>;
}

impl<'a> Icon<'a> {
    /// Creates an `Icon` instance.
    /// # Arguments
    /// * `icon_type` The type of the returned icon.
    /// * `capacity` The target capacity for the underliyng `HashMap<IconOptions, &SourceImage>`.
    /// 
    /// It is important to note that although the returned `Icon` has the capacity specified, the `Icon` will have zero entries.
    /// For an explanation of the difference between length and capacity, see
    /// [*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
    pub fn new(icon_type: IconType, capacity: usize) -> Self {
        Icon { source_map: HashMap::with_capacity(capacity), icon_type }
    }

    /// Creates an `Icon` with the `Ico` icon type.
    /// # Arguments
    /// * `capacity` The target capacity for the underliyng `HashMap<IconOptions, &SourceImage>`.
    /// 
    /// It is important to note that although the returned `Icon` has the capacity specified, the `Icon` will have zero entries.
    /// For an explanation of the difference between length and capacity, see
    /// [*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
    pub fn ico(capacity: usize) -> Self {
        Icon::new(IconType::Ico, capacity)
    }

    /// Creates an `Icon` with the `Icns` icon type.
    /// # Arguments
    /// * `capacity` The target capacity for the underliyng `HashMap<IconOptions, &SourceImage>`.
    /// 
    /// It is important to note that although the returned `Icon` has the capacity specified, the `Icon` will have zero entries.
    /// For an explanation of the difference between length and capacity, see
    /// [*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
    pub fn icns(capacity: usize) -> Self {
        Icon::new(IconType::Icns, capacity)
    }

    /// Creates an `Icon` with the `PngSequece` icon type.
    /// # Arguments
    /// * `capacity` The target capacity for the underliyng `HashMap<IconOptions, &SourceImage>`.
    /// 
    /// It is important to note that although the returned `Icon` has the capacity specified, the `Icon` will have zero entries.
    /// For an explanation of the difference between length and capacity, see
    /// [*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
    pub fn png_sequence(capacity: usize) -> Self {
        Icon::new(IconType::PngSequence, capacity)
    }

    /// Adds an entry to the icon.
    /// 
    /// Returns `Err(Error::SizeAlreadyIncluded(Size))` if any of the sizes listed in `opts.sizes()` is already associated to another entry.
    /// Otherwise returns `Ok(())`.
    pub fn add_entry(&mut self, entry: &Entry, source: &'a SourceImage) -> Result<()> {
        let sizes = self.sizes();

        if self.icon_type == IconType::Ico {
            for &(w, h) in entry {
                if w > MAX_ICO_SIZE || h > MAX_ICO_SIZE || w != h {
                    return Err(Error::InvalidIcoSize((w, h)));
                }
            }
        } else if self.icon_type == IconType::Icns {
            for &size in entry {
                if !VALID_ICNS_SIZES.contains(&size) {
                    return Err(Error::InvalidIcnsSize(size));
                }
            }
        }

        for &size in entry {
            if sizes.contains(&size) {
                return Err(Error::SizeAlreadyIncluded(size));
            }
        }

        self.source_map.insert(entry.clone(), source);

        Ok(())
    }

    /// Remove an entry from the icon.
    /// 
    /// Returns `Some(&SourceImage)` if the icon contains an entry associated with the `opts` argument. Returns `None` otherwise.
    pub fn remove_entry(&mut self, opts: &Entry) -> Option<&SourceImage> {
        self.source_map.remove(opts)
    }

    /// Returns a list of all sizes listed in all icon's entries.
    pub fn sizes(&self) -> Vec<Size> {
        let mut sizes = Vec::with_capacity(self.n_sizes());

        for (entry, _) in &self.source_map {
            for &size in entry {
                sizes.push(size);
            }
        }

        sizes
    }

    /// Returns the total number of sizes in all icon's entries.
    /// 
    /// This method avoids allocating unnecessary resources when accessing `self.sizes().len()`.
    pub fn n_sizes(&self) -> usize {
        self.source_map.iter().fold(0, |sum, (entry, _)| sum + entry.len())
    }

    pub fn rasterize<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(&self, mut resampler: F) -> Result<Vec<RgbaImage>> {
        let mut rasters = Vec::with_capacity(self.n_sizes());

        for (sizes, source) in &self.source_map {
            for &size in sizes {
                match resampler(source, size) {
                    Ok(rasterized) => rasters.push(rasterized),
                    Err(err) => return Err(err)
                }
            }
        }

        Ok(rasters)
    }

    /// Writes the icon to a file or stream.
    pub fn write<W: Write + Seek, F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(&self, w: W, resampler: F) -> Result<()> {
        let rasters = self.rasterize(resampler)?;

        match self.icon_type {
            IconType::Ico =>  write::ico(rasters, w),
            IconType::Icns => write::icns(rasters, w),
            IconType::PngSequence => write::png_sequence(rasters, w)
        }
    }
}

impl<'a> AsRef<SourceMap<'a>> for Icon<'a> {
    fn as_ref(&self) -> &SourceMap<'a> {
        &self.source_map
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

#[cfg(test)]
mod test {
    use crate::{Icon, SourceImage, FromPath};
    use std::fs::File;

    #[test]
    fn test_write() {
        let mut icon = Icon::ico(2);
        let img1 = SourceImage::from_path("test1.svg").unwrap();
        let img2 = SourceImage::from_path("test2.svg").unwrap();

        let _ = icon.add_entry(&entry![(2, 3)], &img1);
        let _ = icon.add_entry(&entry![(3, 2)], &img2);

        let file = File::create("test.ico").unwrap();

        let _ = icon.write(file, crate::resample::linear);
    }
}