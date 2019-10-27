//! A collection of functions to assist in encoding images
//! in commonly used _file formats_.

use crate::{AsSize, Image, EncodingError};
use image::{png::PNGEncoder, ColorType, DynamicImage, GenericImageView};
use std::{io, path::Path};
use resvg::usvg::{Tree, XmlIndent, XmlOptions};

const XML_OPTS: XmlOptions = XmlOptions {
    indent: XmlIndent::None,
    attributes_indent: XmlIndent::None,
    use_single_quote: false,
};

const STD_CAPACITY: usize = 7;

/// A generic representation of an icon encoder.
pub trait Encoder
where
    Self: Sized,
{
    type Key: AsSize + Send + Sync;

    /// Creates a new icon.
    ///
    /// # Example
    /// ```rust
    /// let icon = I::new();
    /// ```
    fn new() -> Self {
        Self::with_capacity(STD_CAPACITY)
    }

    /// Constructs a new, empty `IconEncoder` with the specified capacity.
    /// The `capacity` argument designates the number of entries
    /// that will be allocated.
    ///
    /// # Example
    /// ```rust
    /// let icon = I::with_capacity(5);
    /// ```
    fn with_capacity(capacity: usize) -> Self;

    /// Returns the number of _entries_ contained in the icon.
    fn len(&self) -> usize;

    /// Adds an individual entry to the icon.
    ///
    /// # Arguments
    ///
    /// * `filter` The resampling filter that will be used to re-scale `source`.
    /// * `source` A reference to the source image this entry will be based on.
    /// * `key` Information on the target entry.
    ///
    /// # Return Value
    ///
    /// * Returns `Err(EncodingError::AlreadyIncluded(_))` if the icon already contains
    ///   an entry associated with `key`.
    /// * Returns `Err(EncodingError::Resample(_))` if the resampling filter provided in
    ///   the `filter` argument fails produces results of dimensions other than the
    ///   ones specified by `key`.
    /// * Otherwise returns `Ok(())`.
    fn add_entry<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &Image,
        key: Self::Key,
    ) -> Result<(), EncodingError<Self::Key>>;

    /// Adds a series of entries to the icon.
    ///
    /// # Arguments
    ///
    /// * `filter` The resampling filter that will be used to re-scale `source`.
    /// * `source` A reference to the source image this entry will be based on.
    /// * `keys` A container for the information on the target entries.
    ///
    /// # Return Value
    ///
    /// * Returns `Err(EncodingError::AlreadyIncluded(_))` if the icon already contains an
    ///   entry associated with any of the items of `keys`.
    /// * Returns `Err(EncodingError::Resample(_))` if the resampling filter provided in
    ///   the `filter` argument fails or produces results of dimensions other than the
    ///   ones specified by the items of `keys`.
    /// * Otherwise returns `Ok(())`.
    fn add_entries<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>, I: IntoIterator<Item = Self::Key>>(
        &mut self,
        mut filter: F,
        source: &Image,
        keys: I,
    ) -> Result<(), EncodingError<Self::Key>> {
        for key in keys {
            self.add_entry(|src, size| filter(src, size), source, key)?;
        }

        Ok(())
    }
}

pub trait Write: Encoder {
    /// Writes the contents of the icon to `w`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ikon::encode::{Encoder, Write};
    /// use std::{io, fs::File};
    /// 
    /// fn example<Icon: Write>() -> io::Result<()> {
    ///     let icon = Icon::new();
    ///
    ///     /* Process the icon */
    ///
    ///     let file = File::create("out.icns")?;
    ///     icon.write(file)
    /// }
    /// ```
    fn write<W: io::Write>(&mut self, w: &mut W) -> io::Result<()>;
}

pub trait Save: Encoder {
    /// Writes the contents of the icon to a file on disk.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ikon::encode::{Encoder, Save};
    ///  
    /// fn example<Icon: Save>() -> io::Result<()> {
    ///     let icon = Icon::new();
    ///
    ///     /* Process the icon */
    ///
    ///     icon.save("./output/")
    /// }
    /// ```
    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<()>;
}

/// Converts _raster graphics_ to _PNG_-encoded buffers.
pub fn png(image: &DynamicImage) -> io::Result<Vec<u8>> {
    let data = image.to_rgba().into_raw();
    let mut output = Vec::with_capacity(data.len());

    let encoder = PNGEncoder::new(&mut output);
    encoder.encode(&data, image.width(), image.height(), ColorType::RGBA(8))?;

    Ok(output)
}

#[inline]
/// Converts _vector graphics_ to _UTF8_-encoded _SVG_ strings.
pub fn svg(image: &Tree) -> Vec<u8> {
    image.to_string(XML_OPTS).into_bytes()
}