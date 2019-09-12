//! A collection of commonly used resampling filters.

use crate::{SourceImage, Error};
use std::{io, fmt::Debug};
use image::{imageops, DynamicImage, ImageBuffer, GenericImageView, FilterType, Bgra, ImageError};
use resvg::{usvg::{self, Tree}, raqote::DrawTarget , FitTo};

/// [Linear resampling filter](https://en.wikipedia.org/wiki/Linear_interpolation).
pub fn linear<E: AsRef<u32> + Debug + Eq>(
    source: &SourceImage,
    size: u32
) -> Result<DynamicImage, Error<E>> {
    match source {
        SourceImage::Raster(bit) => Ok(scale(bit, size, FilterType::Triangle)),
        SourceImage::Svg(svg)    => svg_linear(svg, size)
    }
}

/// [Lanczos resampling filter](https://en.wikipedia.org/wiki/Lanczos_resampling).
pub fn cubic<E: AsRef<u32> + Debug + Eq>(
    source: &SourceImage,
    size: u32
) -> Result<DynamicImage, Error<E>> {
    match source {
        SourceImage::Raster(bit) => Ok(scale(bit, size, FilterType::Lanczos3)),
        SourceImage::Svg(svg)    => svg_linear(svg, size)
    }
}

/// [Nearest-Neighbor resampling filter](https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation).
pub fn nearest<E: AsRef<u32> + Debug + Eq>(
    source: &SourceImage,
    size: u32
) -> Result<DynamicImage, Error<E>> {
    match source {
        SourceImage::Raster(bit) => Ok(nearest::resample(bit, size)),
        SourceImage::Svg(svg)    => svg_linear(svg, size)
    }
}

mod nearest {
    use super::{overfit, scale};
    use image::{imageops, DynamicImage, GenericImageView, FilterType};

    pub fn resample(source: &DynamicImage, size: u32) -> DynamicImage {
        let scaled = if source.width() < size as u32 && source.height() < size as u32 {
            scale_integer(source, size)
        } else {
            scale(source, size, FilterType::Nearest)
        };

        overfit(&scaled, size)
    }

    fn scale_integer(source: &DynamicImage, size: u32) -> DynamicImage {
        let (w ,  h) = source.dimensions();

        let scale = if w > h { size / w } else { size / h };
        let (nw, nh) = (w * scale, h * scale);

        DynamicImage::ImageRgba8(imageops::resize(source, nw, nh, FilterType::Nearest))
    }
}

fn scale(source: &DynamicImage, size: u32, filter: FilterType) -> DynamicImage {
    let (w ,  h) = source.dimensions();

    let (nw, nh) = if w > h { (size, (size * h) / w) } else { ((size * w) / h, size) };

    DynamicImage::ImageRgba8(imageops::resize(source, nw, nh, filter))
}

fn overfit(source: &DynamicImage, size: u32) -> DynamicImage {
    let mut output = DynamicImage::new_rgba8(size, size);

    let dx = (output.width()  - source.width() ) / 2;
    let dy = (output.height() - source.height()) / 2;

    imageops::overlay(&mut output, source, dx, dy);
    output
}

fn svg_linear<E: AsRef<u32> + Debug + Eq>(
    source: &Tree,
    size: u32
) -> Result<DynamicImage, Error<E>> {
    let rect = source.svg_node().view_box.rect;
    let (w, h) = (rect.width(), rect.height());
    let fit_to = if w > h { FitTo::Width(size) } else { FitTo::Height(size) };

    let opts = resvg::Options {
        usvg: usvg::Options::default(),
        fit_to,
        background: None
    };

    match resvg::backend_raqote::render_to_image(source, &opts) {
        Some(surface) => draw_target_to_rgba(surface, size),
        None => Err(Error::Io(io::Error::from(io::ErrorKind::AddrNotAvailable)))
    }
}

fn draw_target_to_rgba<E: AsRef<u32> + Debug + Eq>(
    mut surface: DrawTarget,
    size: u32
) -> Result<DynamicImage, Error<E>> {
    let (w, h) = (surface.width() as u32, surface.height() as u32);
    let data = surface.get_data_u8_mut().to_vec();

    match ImageBuffer::<Bgra<u8>, Vec<u8>>::from_vec(w, h, data) {
        Some(buf) => Ok(overfit(&DynamicImage::ImageBgra8(buf), size)),
        None => Err(Error::Image(ImageError::NotEnoughData))
    }
}