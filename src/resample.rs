//! A collection of commonly used resampling filters.

use crate::{SourceImage, Size, Result, Error};
use std::io::{self, Cursor, BufReader};
use image::{imageops, DynamicImage, GenericImageView, FilterType, ImageFormat};
use resvg::{usvg::{self, Tree}, cairo::ImageSurface, FitTo};

/// [Linear resampling filter](https://en.wikipedia.org/wiki/Linear_interpolation).
pub fn linear(source: &SourceImage, size: Size) -> Result<DynamicImage> {
    match source {
        SourceImage::Raster(bit) => Ok(scale(bit, size, FilterType::Triangle)),
        SourceImage::Svg(svg)    => svg_linear(svg, size)
    }
}

/// [Lanczos resampling filter](https://en.wikipedia.org/wiki/Lanczos_resampling).
pub fn cubic(source: &SourceImage, size: Size) -> Result<DynamicImage> {
    match source {
        SourceImage::Raster(bit) => Ok(scale(bit, size, FilterType::Lanczos3)),
        SourceImage::Svg(svg)    => svg_linear(svg, size)
    }
}

/// [Nearest-Neighbor resampling filter](https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation).
pub fn nearest(source: &SourceImage, size: Size) -> Result<DynamicImage> {
    match source {
        SourceImage::Raster(bit) => Ok(nearest::resample(bit, size)),
        SourceImage::Svg(svg)    => svg_linear(svg, size)
    }
}

mod nearest {
    use super::{overfit, scale};
    use crate::Size;
    use image::{imageops, DynamicImage, GenericImageView, FilterType};

    pub fn resample(source: &DynamicImage, size: Size) -> DynamicImage {
        let scaled = if source.width() < size as u32 && source.height() < size as u32 {
            scale_integer(source, size)
        } else {
            scale(source, size, FilterType::Nearest)
        };

        overfit(&scaled, size)
    }

    fn scale_integer(source: &DynamicImage, size: Size) -> DynamicImage {
        let (w ,  h) = source.dimensions();

        let scale = if w > h { size / w } else { size / h };
        let (nw, nh) = (w * scale, h * scale);

        DynamicImage::ImageRgba8(imageops::resize(source, nw, nh, FilterType::Nearest))
    }
}

fn scale(source: &DynamicImage, size: Size, filter: FilterType) -> DynamicImage {
    let (w ,  h) = source.dimensions();

    let (nw, nh) = if w > h { (size, (size * h) / w) } else { ((size * w) / h, size) };

    DynamicImage::ImageRgba8(imageops::resize(source, nw, nh, filter))
}

fn overfit(source: &DynamicImage, size: Size) -> DynamicImage {
    let mut output = DynamicImage::new_rgba8(size, size);

    let dx = (output.width()  - source.width() ) / 2;
    let dy = (output.height() - source.height()) / 2;

    imageops::overlay(&mut output, source, dx, dy);
    output
}

fn svg_linear(source: &Tree, size: Size) -> Result<DynamicImage> {
    let rect = source.svg_node().view_box.rect;
    let (w, h) = (rect.width, rect.height);
    let fit_to = if w > h { FitTo::Width(size) } else { FitTo::Height(size) };

    let opts = resvg::Options {
        usvg: usvg::Options::default(),
        fit_to,
        background: None
    };

    match resvg::backend_cairo::render_to_image(source, &opts) {
        Some(surface) => cairo_surface_to_rgba(&surface, size),
        None => Err(Error::Io(io::Error::from(io::ErrorKind::AddrNotAvailable)))
    }
}

fn cairo_surface_to_rgba(surface: &ImageSurface, size: Size) -> Result<DynamicImage> {
    let len = surface.get_stride() * surface.get_height();
    let mut data = Vec::with_capacity(len as usize);
    surface.write_to_png(&mut data)?;

    let output = image::load(BufReader::new(Cursor::new(data)), ImageFormat::PNG)?;
    Ok(overfit(&output, size))
}