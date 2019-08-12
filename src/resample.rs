use crate::{SourceImage, Size, Result, Error};
use nsvg::{image::{imageops, RgbaImage, FilterType}, SvgImage};

/// [Linear resampling filter](https://en.wikipedia.org/wiki/Linear_interpolation).
pub fn linear(source: &SourceImage, size: Size) -> Result<RgbaImage> {
    match source {
        SourceImage::Bitmap(bit) => Ok(imageops::resize(bit, size, size, FilterType::Triangle)),
        SourceImage::Svg(svg) => svg_linear(svg, size)
    }
}

/// [Nearest-Neighbor resampling filter](https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation).
pub fn nearest_neighbor(source: &SourceImage, size: Size) -> Result<RgbaImage> {
    match source {
        SourceImage::Bitmap(bit) => Ok(imageops::resize(bit, size, size, FilterType::Nearest)),
        SourceImage::Svg(svg) => svg_linear(svg, size)
    }
}

/// [Lanczos resampling filter](https://en.wikipedia.org/wiki/Lanczos_resampling).
pub fn cubic(source: &SourceImage, size: Size) -> Result<RgbaImage> {
    match source {
        SourceImage::Bitmap(bit) => Ok(imageops::resize(bit, size, size, FilterType::Lanczos3)),
        SourceImage::Svg(svg) => svg_linear(svg, size)
    }
}

fn svg_linear(svg: &SvgImage, size: u32) -> Result<RgbaImage> {  
    match svg.rasterize((size as f32) / svg.width()) {
        Ok(raster) => Ok(imageops::resize(&raster, raster.width(), size, FilterType::Triangle)),
        Err(err) => Err(Error::Nsvg(err))
    }
}