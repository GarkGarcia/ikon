use crate::{SourceImage, Size, Result, Error};
use nsvg::{image::{imageops, RgbaImage, FilterType}, SvgImage};

/// [Linear resampling filter](https://en.wikipedia.org/wiki/Linear_interpolation).
pub fn linear(source: &SourceImage, size: Size) -> Result<RgbaImage> {
    let (w, h) = size;

    match source {
        SourceImage::Bitmap(bit) => Ok(imageops::resize(bit, w as u32, h as u32, FilterType::Triangle)),
        SourceImage::Svg(svg) => svg_linear(svg, f32::from(w), h as u32)
    }
}

/// [Nearest-Neighbor resampling filter](https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation).
pub fn nearest_neighbor(source: &SourceImage, size: Size) -> Result<RgbaImage> {
    let (w, h) = size;

    match source {
        SourceImage::Bitmap(bit) => Ok(imageops::resize(bit, w as u32, h as u32, FilterType::Nearest)),
        SourceImage::Svg(svg) => svg_linear(svg, f32::from(w), h as u32)
    }
}

/// [Lanczos resampling filter](https://en.wikipedia.org/wiki/Lanczos_resampling).
pub fn cubic(source: &SourceImage, size: Size) -> Result<RgbaImage> {
    let (w, h) = size;

    match source {
        SourceImage::Bitmap(bit) => Ok(imageops::resize(bit, w as u32, h as u32, FilterType::Lanczos3)),
        SourceImage::Svg(svg) => svg_linear(svg, f32::from(w), h as u32)
    }
}

fn svg_linear(svg: &SvgImage, w: f32, h: u32) -> Result<RgbaImage> {  
    match svg.rasterize(w / svg.width()) {
        Ok(raster) => Ok(imageops::resize(&raster, raster.width(), h, FilterType::Triangle)),
        Err(err) => Err(Error::Nsvg(err))
    }
}