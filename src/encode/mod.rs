//! Traits, types and functions to assist in encoding commonly used 
//! _icon formats_.

use crate::{Icon, Image};
use image::{DynamicImage, ImageOutputFormat, ImageError};
use std::{io::{self, BufWriter}, path::Path, fs::File};
use resvg::usvg::{Tree, XmlIndent, XmlOptions};
pub use error::EncodingError;

mod error;

const XML_OPTS: XmlOptions = XmlOptions {
    indent: XmlIndent::None,
    attributes_indent: XmlIndent::None,
    use_single_quote: false,
};

/// The `Encode` trait represents a generic _icon family_ encoder, providing 
/// basic inicialization methods as well as functionality for adding _icons_.
/// 
/// # Example
/// 
/// In this example we'll create a very simple `Encode` implementor whose
/// icons are _positive integers_. First of all, we'll need a `Icon` type:
/// 
/// ```rust
/// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// pub struct Icon(pub u16);
/// 
/// impl ikon::Icon for Icon {
///     fn size(&self) -> (u32, u32) {
///         match self {
///             Icon(0) => (256, 256),
///             Icon(size) => (*size as u32, *size as u32)
///         }
///     }
/// }
/// ```
/// 
/// Note that `Icon(0)` represents `Icon(256)`. We can then implement our `IconFamily` type:
/// 
/// ```rust
/// use std::{io, marker, collections::hash_map::{HashMap, Entry}};
/// use ikon::{encode::{Encode, EncodingError}, image::DynamicImage, Image};
///
/// #[derive(Clone)]
/// pub struct IconFamily<Icon: ikon::Icon + Send + Sync> {
///     internal: HashMap<(u32, u32), DynamicImage>,
///     phantom: marker::PhantomData<Icon>
/// }
/// 
/// impl<Icon: ikon::Icon + Send + Sync> Encode for IconFamily<Icon> {
///     type Icon = Icon;
///
///     #[inline]
///     fn len(&self) -> usize {
///         self.internal.len()
///     }
/// 
///     fn add_icon<F: FnMut(&DynamicImage, (u32, u32)) -> io::Result<DynamicImage>>(
///         &mut self,
///         filter: F,
///         source: &Image,
///         icon: Self::Icon,
///     ) -> Result<&mut Self, EncodingError<Self::Icon>> {
///         let size = icon.size();
/// 
///         if let Entry::Vacant(entry) = self.internal.entry(size) {
///             entry.insert(source.rasterize(filter, size)?);
///             Ok(self)
///         } else {
///             Err(EncodingError::AlreadyIncluded(icon))
///         }
///     }
/// }
/// ```
pub trait Encode: Sized {
    type Icon: Icon + Send + Sync;

    /// Returns the number of _icons_ contained in the icon.
    fn len(&self) -> usize;

    /// Adds an individual icon to the icon family.
    ///
    /// # Arguments
    ///
    /// * `filter` The resampling filter that will be used to re-scale `source`.
    /// * `source` A reference to the source image this icon will be based on.
    /// * `icon` Information on the target icon.
    ///
    /// # Return Value
    ///
    /// * Returns `Err(EncodingError::AlreadyIncluded(_))` if the icon family
    ///   already contains `icon`.
    /// * Returns `Err(EncodingError::Resample(_))` if the resampling filter 
    ///   provided in the `filter` argument fails produces results of 
    ///   dimensions other than the ones specified by `icon`.
    /// * Otherwise returns `Ok(())`.
    fn add_icon<F: FnMut(&DynamicImage, (u32, u32)) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &Image,
        icon: Self::Icon,
    ) -> Result<&mut Self, EncodingError<Self::Icon>>;

    /// Adds a series of icons to the icon family.
    ///
    /// # Arguments
    ///
    /// * `filter` The resampling filter that will be used to re-scale `source`.
    /// * `source` A reference to the source image this icon will be based on.
    /// * `icons` A container for the information on the target icons.
    ///
    /// # Return Value
    ///
    /// * Returns `Err(EncodingError::AlreadyIncluded(_))` if the icon family
    ///   already contains any of the items of `icons`.
    /// * Returns `Err(EncodingError::Resample(_))` if the resampling filter 
    ///   provided in the `filter` argument fails or produces results of 
    ///   dimensions other than the ones specified by the items of `icons`.
    /// * Otherwise returns `Ok(())`.
    fn add_icons<
        F: FnMut(&DynamicImage, (u32, u32)) -> io::Result<DynamicImage>,
        I: IntoIterator<Item = Self::Icon>
    >(
        &mut self,
        mut filter: F,
        source: &Image,
        icons: I,
    ) -> Result<&mut Self, EncodingError<Self::Icon>> {
        for icon in icons {
            self.add_icon(|src, size| filter(src, size), source, icon)?;
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
    /// Writes the contents of the icon family to `w`.
    fn write<W: io::Write>(&mut self, w: &mut W) -> io::Result<&mut Self>;
}

/// The `Save` trait provides functionality for saving the
/// contents of an `Encode` to the local file system.
/// 
/// Usefull for _icon formats_ such as _favicon_.
pub trait Save: Encode {
    /// Writes the contents of the icon family to disk.
    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<&mut Self>;
}

impl<T: Write> Save for T {
    #[inline]
    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<&mut Self> {
        let mut file = BufWriter::new(File::create(path)?);
        self.write(&mut file)
    }
}

#[inline]
/// Converts _raster graphics_ to _PNG_-encoded buffers.
pub fn png<W: io::Write>(image: &DynamicImage, w: &mut W) -> io::Result<()> {
    image
        .write_to(w, ImageOutputFormat::PNG)
        .map_err(image_err_to_io)
}

#[inline]
/// Converts _raster graphics_ to _BMP_-encoded buffers.
pub fn bmp<W: io::Write>(image: &DynamicImage, w: &mut W) -> io::Result<()> {
    image
        .write_to(w, ImageOutputFormat::BMP)
        .map_err(image_err_to_io)
}

#[inline]
/// Converts _vector graphics_ to _UTF8_-encoded _SVG_ strings.
pub fn svg<W: io::Write>(image: &Tree, w: &mut W) -> io::Result<()> {
    w.write_all(image.to_string(XML_OPTS).as_ref())
}

#[inline]
/// Convert an `ImageError` to an `io::Error`
fn image_err_to_io(err: ImageError) -> io::Error {
    match err {
        ImageError::IoError(err) => err,
        _ => unreachable!()
    }
}

