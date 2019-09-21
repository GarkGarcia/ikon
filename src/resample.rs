//! A collection of commonly used resampling filters.

use crate::{SourceImage, Entry, Error};
use std::fmt::Debug;
use image::{imageops, DynamicImage, ImageBuffer, GenericImageView, FilterType, Bgra};
use resvg::{usvg::{self, Tree}, raqote::DrawTarget , FitTo};

/// [Linear resampling filter](https://en.wikipedia.org/wiki/Linear_interpolation).
pub fn linear(source: &SourceImage, size: u32) -> DynamicImage {
    match source {
        SourceImage::Raster(bit) => scale(bit, size, FilterType::Triangle),
        SourceImage::Svg(svg)    => svg_linear(svg, size)
    }
}

/// [Lanczos resampling filter](https://en.wikipedia.org/wiki/Lanczos_resampling).
pub fn cubic(source: &SourceImage, size: u32) -> DynamicImage {
    match source {
        SourceImage::Raster(bit) => scale(bit, size, FilterType::Lanczos3),
        SourceImage::Svg(svg)    => svg_linear(svg, size)
    }
}

/// [Nearest-Neighbor resampling filter](https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation).
pub fn nearest(source: &SourceImage, size: u32) -> DynamicImage {
    match source {
        SourceImage::Raster(bit) => nearest_raster(bit, size),
        SourceImage::Svg(svg)    => svg_linear(svg, size)
    }
}

fn nearest_raster(source: &DynamicImage, size: u32) -> DynamicImage {
    let scaled = if source.width() < size && source.height() < size {
        nearest_upscale_integer(source, size)
    } else {
        scale(source, size, FilterType::Nearest)
    };

    overfit(&scaled, size)
}

fn nearest_upscale_integer(source: &DynamicImage, size: u32) -> DynamicImage {
    let (w ,  h) = source.dimensions();

    let scale = if w > h { size / w } else { size / h };
    let (nw, nh) = (w * scale, h * scale);

    DynamicImage::ImageRgba8(imageops::resize(source, nw, nh, FilterType::Nearest))
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

fn svg_linear(source: &Tree, size: u32) -> DynamicImage {
    let rect = source.svg_node().view_box.rect;
    let (w, h) = (rect.width(), rect.height());
    let fit_to = if w > h { FitTo::Width(size) } else { FitTo::Height(size) };

    let opts = resvg::Options {
        usvg: usvg::Options::default(),
        fit_to,
        background: None
    };

    // In this context it's safe to assume render_to_image will return Some(_)
    // https://github.com/RazrFalcon/resvg/issues/175#issuecomment-531477376
    let draw_target = resvg::backend_raqote::render_to_image(source, &opts)
        .expect("Could not render svg tree to image buffer");

    draw_target_to_rgba(draw_target, size)
}

#[inline]
fn draw_target_to_rgba(mut surface: DrawTarget, size: u32) -> DynamicImage {
    let (w, h) = (surface.width() as u32, surface.height() as u32);
    let data = surface.get_data_u8_mut().to_vec();

    // If ImageBuffer::from_vec returns None then there's a bug in
    // resvg
    match ImageBuffer::<Bgra<u8>, Vec<u8>>::from_vec(w, h, data) {
        Some(buf) => overfit(&DynamicImage::ImageBgra8(buf), size),
        None      => panic!("Buffer in not big enought")
    }
}

/// Aplies a resampling filter to a source and checks if the outputted dimensions match
/// the ones specified by `size`.
pub(crate) fn safe_filter<F: FnMut(&SourceImage, u32) -> DynamicImage, E: Entry + Debug + Eq>(
    mut filter: F,
    source: &SourceImage,
    size: u32
) -> Result<DynamicImage, Error<E>> {
    let icon = filter(source, size);
    let (icon_w, icon_h) = icon.dimensions();

    if icon_w != size || icon_h != size {
        Err(Error::MismatchedDimensions(size, (icon_w, icon_h)))
    } else {
        Ok(icon)
    }
}