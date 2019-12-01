//! A collection of commonly used resampling filters.

use std::io;
use image::{imageops, DynamicImage, ImageBuffer, GenericImageView, FilterType, Bgra};
use resvg::{usvg::{self, Tree}, raqote::DrawTarget , FitTo};
pub use error::ResampleError;

mod error;

/// [Linear resampling filter](https://en.wikipedia.org/wiki/Linear_interpolation).
pub fn linear(source: &DynamicImage, size: (u32, u32)) -> io::Result<DynamicImage> {
    overfit(&scale(source, size, FilterType::Triangle)?, size)
}

/// [Lanczos resampling filter](https://en.wikipedia.org/wiki/Lanczos_resampling).
pub fn cubic(source: &DynamicImage, size: (u32, u32)) -> io::Result<DynamicImage> {
    overfit(&scale(source, size, FilterType::Lanczos3)?, size)
}

/// [Nearest-Neighbor resampling filter](https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation).
pub fn nearest(source: &DynamicImage, size: (u32, u32)) -> io::Result<DynamicImage> {
    let scaled = if source.width() < size.0 && source.height() < size.1 {
        nearest_upscale_integer(source, size)?
    } else {
        scale(source, size, FilterType::Nearest)?
    };

    overfit(&scaled, size)
}

/// Aplies a resampling filter to `source` and checks if the dimensions
/// of the output match the ones specified by `size`.
pub fn apply<F: FnMut(&DynamicImage, (u32, u32)) -> io::Result<DynamicImage>>(
    mut filter: F,
    source: &DynamicImage,
    size: (u32, u32)
) -> Result<DynamicImage, ResampleError> {
    let icon = filter(source, size)?;
    let dims = icon.dimensions();

    if dims != size {
        Err(ResampleError::MismatchedDimensions(size, dims))
    } else {
        Ok(icon)
    }
}

/// Rescales `source` to fit the dimensions specified by `size` while only scaling it on an integer scale.
fn nearest_upscale_integer(source: &DynamicImage, size: (u32, u32)) -> io::Result<DynamicImage> {
    let (w, h) = source.dimensions();

    let scale = if w > h { size.0 / w } else { size.1 / h };
    let (nw, nh) = (w * scale, h * scale);

    Ok(DynamicImage::ImageRgba8(imageops::resize(source, nw, nh, FilterType::Nearest)))
}

/// Rescales `source` to fit the dimensions specified by `size`.
fn scale(source: &DynamicImage, size: (u32, u32), filter: FilterType) -> io::Result<DynamicImage> {
    let (w, h) = source.dimensions();
    let (nw, nh) = if w > h { (size.0, size.0 * h / w)} else { (size.1 * w / h, size.1) };

    Ok(DynamicImage::ImageRgba8(imageops::resize(source, nw, nh, filter)))
}

/// Adds transparent borders to an image so that the output is square.
fn overfit(source: &DynamicImage, size: (u32, u32)) -> io::Result<DynamicImage> {
    let mut output = DynamicImage::new_rgba8(size.0, size.1);

    let dx = (output.width()  - source.width() ) / 2;
    let dy = (output.height() - source.height()) / 2;

    imageops::overlay(&mut output, source, dx, dy);
    Ok(output)
}

/// Rasterizes an _SVG_ tree to a `DynamicImage`.
pub(crate) fn svg(source: &Tree, size: (u32, u32)) -> Result<DynamicImage, ResampleError> {
    let rect = source.svg_node().view_box.rect;
    let (w, h) = (rect.width(), rect.height());
    let fit_to = if w > h { FitTo::Width(size.0) } else { FitTo::Height(size.1) };

    let opts = resvg::Options {
        usvg: usvg::Options::default(),
        fit_to,
        background: None
    };

    // In this context it's safe to assume render_to_image will return Some(_)
    // https://github.com/RazrFalcon/resvg/issues/175#issuecomment-531477376
    let draw_target = resvg::backend_raqote::render_to_image(source, &opts)
        .expect("Could not render svg tree to image buffer");

    Ok(draw_target_to_rgba(draw_target, size)?)
}

#[inline]
/// Converts a `DrawTarget` to a `DynamicImage`.
fn draw_target_to_rgba(mut surface: DrawTarget, size: (u32, u32)) -> io::Result<DynamicImage> {
    let (w, h) = (surface.width() as u32, surface.height() as u32);
    let data = surface.get_data_u8_mut().to_vec();

    // If ImageBuffer::from_vec returns None then there's a bug in
    // resvg
    match ImageBuffer::<Bgra<u8>, Vec<u8>>::from_vec(w, h, data) {
        Some(buf) => overfit(&DynamicImage::ImageBgra8(buf), size),
        None      => panic!("Buffer in not big enought")
    }
}