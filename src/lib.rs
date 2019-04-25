//! A simple solution for generating `.ico` and `.icns` icons. The crate also supports png sequences as an output format.

extern crate zip;
extern crate png_encode_mini;
extern crate ico;
extern crate icns;
pub extern crate nsvg;

mod write;

use std::{convert::From, path::Path, marker::Sized, io::{self, Write, Seek}, default::Default, collections::HashMap};
use nsvg::{image::{DynamicImage, RgbaImage, GenericImage, FilterType}, SvgImage};
use zip::result::ZipError;
pub use nsvg::image::{self, imageops};

const MAX_ICO_SIZE: u16 = 265;
const VALID_ICNS_SIZES: [(u16, u16);7] = [(16, 16), (32, 32), (64, 64), (128, 128), (256, 256), (512, 512), (1024, 1024)];

pub type Size = (u16, u16);
pub type SourceMap<'a> = HashMap<IconOptions, &'a SourceImage>;
pub type Result<T> = std::result::Result<T, Error>;

/// A generic representation of an icon.
pub struct Icon<'a> {
    source_map: SourceMap<'a>,
    icon_type: IconType
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// A representation of an entry's properties.
pub struct IconOptions {
    sizes: Vec<Size>,
    pub filter: ResamplingFilter,
    pub crop: Crop
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResamplingFilter {
    Neareast,
    Linear,
    Cubic
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Crop {
    Proportional,
    Square
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
pub trait FromFile where Self: Sized {
    /// Constructs `Self` from a given path.
    fn from_file<P: AsRef<Path>>(path: P) -> Option<Self>;
}

/// Rasterizes a generic image to series of `RgbaImage`'s, conforming to the configuration options specifyed in the `options` argument.
pub trait Raster<E> {
    /// Rasterizes `self` to series of `RgbaImage`'s, conforming to the configuration options specifyed in the `options` argument.
    /// 
    /// Returns `Ok(Vec<RgbaImage>)` if the rasterazation was sucessfull. Otherwise returns `Err<E>`
    fn raster(&self, opts: &IconOptions) -> std::result::Result<Vec<RgbaImage>, E>;
}

impl<'a> Icon<'a> {
    /// Creates an `Icon` with the `Ico` icon type.
    /// # Arguments
    /// * `capacity` The target capacity for the underliyng `HashMap<IconOptions, &SourceImage>`.
    /// 
    /// It is important to note that although the returned `Icon` has the capacity specified, the `Icon` will have zero entries.
    /// For an explanation of the difference between length and capacity, see
    /// [*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
    pub fn ico(capacity: usize) -> Self {
        Icon { source_map: HashMap::with_capacity(capacity), icon_type: IconType::Ico }
    }

    /// Creates an `Icon` with the `Icns` icon type.
    /// # Arguments
    /// * `capacity` The target capacity for the underliyng `HashMap<IconOptions, &SourceImage>`.
    /// 
    /// It is important to note that although the returned `Icon` has the capacity specified, the `Icon` will have zero entries.
    /// For an explanation of the difference between length and capacity, see
    /// [*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
    pub fn icns(capacity: usize) -> Self {
        Icon { source_map: HashMap::with_capacity(capacity), icon_type: IconType::Icns }
    }

    /// Creates an `Icon` with the `PngSequece` icon type.
    /// # Arguments
    /// * `capacity` The target capacity for the underliyng `HashMap<IconOptions, &SourceImage>`.
    /// 
    /// It is important to note that although the returned `Icon` has the capacity specified, the `Icon` will have zero entries.
    /// For an explanation of the difference between length and capacity, see
    /// [*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).
    pub fn png_sequence(capacity: usize) -> Self {
        Icon { source_map: HashMap::with_capacity(capacity), icon_type: IconType::PngSequence }
    }

    /// Adds an entry to the icon.
    /// 
    /// Returns `Err(Error::SizeAlreadyIncluded(Size))` if any of the sizes listed in `opts.sizes()` is already associated to another entry.
    /// Otherwise returns `Ok(())`.
    pub fn add_entry(&mut self, opts: IconOptions, source: &'a SourceImage) -> Result<()> {
        let sizes = self.sizes();

        match self.icon_type {
            IconType::Ico => for (w, h) in opts.sizes() {
                if w > MAX_ICO_SIZE || h > MAX_ICO_SIZE || w != h {
                    return Err(Error::InvalidIcoSize((w, h)));
                }
            },
            IconType::Icns => for size in opts.sizes() {
                if !VALID_ICNS_SIZES.contains(&size) {
                    return Err(Error::InvalidIcnsSize(size));
                }
            },
            IconType::PngSequence => {}
        }

        for size in opts.sizes() {
            if sizes.contains(&size) {
                return Err(Error::SizeAlreadyIncluded(size));
            }
        }

        self.source_map.insert(opts, source);

        Ok(())
    }

    /// Remove an entry from the icon.
    /// 
    /// Returns `Some(&SourceImage)` if the icon contains an entry associated with the `opts` argument. Returns `None` otherwise.
    pub fn remove_entry(&mut self, opts: &IconOptions) -> Option<&SourceImage> {
        self.source_map.remove(opts)
    }

    /// Returns a list of all sizes listed in all icon's entries.
    pub fn sizes(&self) -> Vec<Size> {
        let mut sizes = Vec::with_capacity(self.n_sizes());

        for (opt, _) in &self.source_map {
            let mut opt_sizes = opt.sizes().clone();
            sizes.append(&mut opt_sizes);
        }

        sizes
    }

    /// Returns the total number of sizes in all icon's entries.
    /// 
    /// This method avoids allocating unnecessary resources when accessing `self.sizes().len()`.
    pub fn n_sizes(&self) -> usize {
        self.source_map.iter().fold(0, |sum, (opts, _)| sum + opts.n_sizes())
    }

    pub fn raster(&self) -> Result<Vec<RgbaImage>> {
        let mut rasters = Vec::with_capacity(self.n_sizes());

        for (opts, source) in &self.source_map {
            match source {
                SourceImage::Svg(svg) => rasters.append(&mut svg.raster(&opts)?),
                SourceImage::Bitmap(bit) => rasters.append(&mut bit.raster(&opts)?)
            }
        }

        Ok(rasters)
    }

    /// Writes the icon to a file or stream.
    pub fn write<W: Write + Seek>(&self, w: W) -> Result<()> {
        let rasters = self.raster()?;

        match self.icon_type {
            IconType::Ico => write::ico(rasters, w),
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

impl IconOptions {
    /// Constructs a new `IconOptions`.
    pub fn new(
        sizes: Vec<Size>,
        filter: ResamplingFilter,
        crop: Crop
    ) -> Self {
        IconOptions { sizes, filter, crop }
    }

    /// Returns a copy of `self.sizes`.
    pub fn sizes(&self) -> Vec<Size> {
        self.sizes.clone()
    }

    /// Returns the lenght of `self.sizes`.
    /// 
    /// This method avoids allocating unnecessary resources when accessing `self.sizes().len()`.
    pub fn n_sizes(&self) -> usize {
        self.sizes.len()
    }
}

impl Default for IconOptions {
    fn default() -> Self {
        IconOptions { sizes: Vec::new(), filter: ResamplingFilter::Neareast, crop: Crop::Square }
    }
}

impl ResamplingFilter {
    pub fn from(filter: FilterType) -> Option<Self> {
        match filter {
            FilterType::Nearest    => Some(ResamplingFilter::Neareast),
            FilterType::Triangle   => Some(ResamplingFilter::Linear),
            FilterType::CatmullRom => Some(ResamplingFilter::Cubic),
            _ => None
        }
    }
}

impl Into<FilterType> for ResamplingFilter {
    fn into(self) -> FilterType {
        match self {
            ResamplingFilter::Neareast => FilterType::Nearest,
            ResamplingFilter::Linear   => FilterType::Triangle,
            ResamplingFilter::Cubic    => FilterType::CatmullRom
        }
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

impl FromFile for SourceImage {
    fn from_file<P: AsRef<Path>>(path: P) -> Option<Self> {
        if let Ok(din) = image::open(&path) {
            Some(SourceImage::Bitmap(din))
        } else if let Ok(svg) = nsvg::parse_file(path.as_ref(), nsvg::Units::Pixel, 96.0) {
            Some(SourceImage::Svg(svg))
        } else {
            None
        }
    }
}

impl Raster<Error> for SvgImage {
    fn raster(&self, opts: &IconOptions) -> Result<Vec<RgbaImage>> {
        let mut images = Vec::with_capacity(opts.n_sizes());

        for (w, h) in opts.sizes() {
            match self.rasterize(f32::from(w) / self.width()) {
                Ok(buf) => if opts.crop == Crop::Square && (w as u32 != buf.width() || h as u32 != buf.height()) {
                    let din = DynamicImage::ImageRgba8(buf);
                    let reframed = reframe(&din, w as u32, h as u32);

                    images.push(reframed);
                } else {
                    images.push(buf);
                },
                Err(err) => match err {
                    nsvg::Error::IoError(err) => return Err(Error::Io(err)),
                    err => return Err(Error::Nsvg(err))
                }
            }
        }

        Ok(images)
    }
}

impl Raster<Error> for DynamicImage {
    fn raster(&self, opts: &IconOptions) -> Result<Vec<RgbaImage>> {
        let mut rasters = Vec::with_capacity(opts.n_sizes());

        for (w, h) in opts.sizes() {
            let reframed = reframe(&self.resize(w as u32, h as u32, opts.filter.into()), w as u32, h as u32);
            rasters.push(reframed);
        }

        Ok(rasters)
    }
}

fn reframe(source: &DynamicImage, w: u32, h: u32) -> RgbaImage {
    if source.width() == w && source.height() == h {
        source.to_rgba()
    } else {
        let mut output = DynamicImage::new_rgba8(w, h);
        let dx = (output.width() - source.width()) / 2;
        let dy = (output.height() - source.height()) / 2;

        imageops::overlay(&mut output, &source, dx, dy);
        output.to_rgba()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
