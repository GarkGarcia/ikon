//! Traits, types and functions to assist in encoding commonly used _icon formats_.

use crate::{AsSize, Image};
use image::{png::PNGEncoder, bmp::BMPEncoder, ColorType, DynamicImage, GenericImageView};
use std::{io::{self, BufWriter}, path::Path, fs::File};
use resvg::usvg::{Tree, XmlIndent, XmlOptions};
pub use error::EncodingError;

mod error;

const XML_OPTS: XmlOptions = XmlOptions {
    indent: XmlIndent::None,
    attributes_indent: XmlIndent::None,
    use_single_quote: false,
};

const STD_CAPACITY: usize = 7;

/// The `Encode` trait represents a generic icon encoder, providing basic
/// inicialization methods as well as functionality for adding _entries_.
/// 
/// # Example
/// 
/// In this example we'll create a very simple `Encode` implementor whose
/// keys are _positive integers_. First of all, we'll need a `Key` type:
/// 
/// ```rust
/// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// pub struct Key(pub u16);
/// 
/// impl AsSize for Key {
///     fn as_size(&self) -> u32 {
///         if self.0 == 0 {
///             256
///         } else {
///             *self.0
///         }
///     }
/// }
/// ```
/// 
/// Note that `Key(0)` represents `Key(256)`. We can then implement our `Icon` type:
/// 
/// ```rust
/// #[derive(Clone)]
/// pub struct Icon {
///     internal: HashMap<u16, DynamicImage>
/// }
/// 
/// impl Encode for Icon {
///     type Key = Key;
/// 
///     fn with_capacity(capacity: usize) -> Self {
///         Self { internal: HashMap::with_capacity(capacity) }
///     }
/// 
///     fn add_entry<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>>(
///         &mut self,
///         filter: F,
///         source: &Image,
///         key: Self::Key,
///     ) -> Result<(), EncodingError<Self::Key>> {
///         let size = key.as_size();
/// 
///         if let Entry::Vacant(entry) = self.internal.entry(size) {
///             entry.insert(source.rasterize(filter, size));
///             Ok(())
///         } else {
///             Err(EncodingError::AlreadyIncluded(key))
///         }
///     }
/// }
/// ```
pub trait Encode: Sized {
    type Key: AsSize + Send + Sync;

    /// Creates a new icon.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let icon = Icon::new();
    /// ```
    fn new() -> Self {
        Self::with_capacity(STD_CAPACITY)
    }

    /// Constructs a new, empty `IconEncoder` with the specified capacity.
    /// The `capacity` argument designates the number of entries
    /// that will be allocated.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let icon = Icon::with_capacity(5);
    /// ```
    fn with_capacity(capacity: usize) -> Self;

    /// Returns the number of _entries_ contained in the icon.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let len = icon.len();
    /// ```
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
    /// 
    /// # Example
    /// 
    /// ```rust
    /// fn main() -> io::Result<()> {
    ///     let image = Image::open("image.svg")?;
    ///     let icon = Icon::new();
    /// 
    ///     icon.add_entry(resample::linear, image, Key(32))?
    ///         .add_entry(resample::nearest, image, Key(64))?;
    /// 
    ///     Ok(())
    /// }
    /// ```
    fn add_entry<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &Image,
        key: Self::Key,
    ) -> Result<&mut Self, EncodingError<Self::Key>>;

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
    /// 
    /// # Example
    /// 
    /// ```rust
    /// fn main() -> io::Result<()> {
    ///     let image = Image::open("image.svg")?;
    ///     let icon = Icon::new();
    /// 
    ///     icon.add_entries(
    ///         resample::linear,
    ///         image,
    ///         vec![Key(32), Key(64), Key(128)]
    ///     )?;
    /// 
    ///     Ok(())
    /// }
    /// ```
    fn add_entries<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>, I: IntoIterator<Item = Self::Key>>(
        &mut self,
        mut filter: F,
        source: &Image,
        keys: I,
    ) -> Result<&mut Self, EncodingError<Self::Key>> {
        for key in keys {
            self.add_entry(|src, size| filter(src, size), source, key)?;
        }

        Ok(self)
    }
}

/// The `Write` trait provides functionality for writing the
/// contents of an `Encode` into a `io::Write` implementor.
/// 
/// Usefull for _icon formats_ such as `.ico` and `.icns`
/// files.
pub trait Write: Encode {
    /// Writes the contents of the icon to `w`.
    ///
    /// # Example
    ///
    /// ```rust
    /// fn main() -> io::Result<()> {
    ///     let icon = Icon::new();
    ///
    ///     /* Process the icon */
    ///
    ///     let file = File::create("out.icns")?;
    ///     icon.write(file)
    /// }
    /// ```
    fn write<W: io::Write>(&mut self, w: &mut W) -> io::Result<&mut Self>;
}

/// The `Save` trait provides functionality for saving the
/// contents of an `Encode` to the local file system.
/// 
/// Usefull for _icon formats_ such as _favicon_.
pub trait Save: Encode {
    /// Writes the contents of the icon to disk.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ikon::encode::{Encode, Save};
    ///  
    /// fn main() -> io::Result<()> {
    ///     let icon = Icon::new();
    ///
    ///     /* Process the icon */
    ///
    ///     icon.save("./output/")
    /// }
    /// ```
    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<&mut Self>;
}

impl<T: Write> Save for T {
    #[inline]
    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<&mut Self> {
        let mut file = BufWriter::new(File::create(path)?);
        self.write(&mut file)
    }
}

/// Converts _raster graphics_ to _PNG_-encoded buffers.
pub fn png(image: &DynamicImage) -> io::Result<Vec<u8>> {
    let data = image.to_rgba().into_raw();
    let mut output = Vec::with_capacity(data.len());

    let encoder = PNGEncoder::new(&mut output);
    encoder.encode(&data, image.width(), image.height(), ColorType::RGBA(8))?;

    Ok(output)
}

/// Converts _raster graphics_ to _BMP_-encoded buffers.
pub fn bmp(image: &DynamicImage) -> io::Result<Vec<u8>> {
    let data = image.to_rgba().into_raw();
    let mut output = Vec::with_capacity(data.len());

    let mut encoder = BMPEncoder::new(&mut output);
    encoder.encode(&data, image.width(), image.height(), ColorType::RGBA(8))?;

    Ok(output)
}

#[inline]
/// Converts _vector graphics_ to _UTF8_-encoded _SVG_ strings.
pub fn svg(image: &Tree) -> Vec<u8> {
    image.to_string(XML_OPTS).into_bytes()
}