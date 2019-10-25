//! A collection of functions to assist in encoding images
//! in commonly used _file formats_.

use image::{png::PNGEncoder, ColorType, DynamicImage, GenericImageView};
use std::io;
use resvg::usvg::{Tree, XmlIndent, XmlOptions};

const XML_OPTS: XmlOptions = XmlOptions {
    indent: XmlIndent::None,
    attributes_indent: XmlIndent::None,
    use_single_quote: false,
};

/// Encodes _raster graphics_ in _PNG_ buffers.
pub fn png(image: &DynamicImage) -> io::Result<Vec<u8>> {
    let data = image.to_rgba().into_raw();
    let mut output = Vec::with_capacity(data.len());

    let encoder = PNGEncoder::new(&mut output);
    encoder.encode(&data, image.width(), image.height(), ColorType::RGBA(8))?;

    Ok(output)
}

#[inline]
/// Converts _vector graphics_ in UTF-8 encoded _SVG_ buffers.
pub fn svg(image: &Tree) -> Vec<u8> {
    image.to_string(XML_OPTS).into_bytes()
}