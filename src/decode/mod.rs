//! Traits, types and functions to assist in dencoding commonly used _icon formats_.

use crate::{load_raster, load_vector, AsSize, Image};
pub use error::DecodingError;
use std::{io::{self, Read, Seek}};
use image::{ImageFormat, DynamicImage};
use resvg::usvg::Tree;

mod error;

/// The `Decode` trait represents a generic icon decoder, providing methods
/// for generating icons from byte streams, as well as functionality querying
/// and inspecting _entries_.
/// 
/// # Example
/// 
/// In this example we'll create a very simple `Decode` implementor whose
/// keys are _positive integers_. First of all, we'll need a `Key` type:
/// 
/// ```rust
/// #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
/// Note that `Key(0)` represents `Key(256)`. We can then implement our `Icon` type.
/// 
/// ```rust
/// #[derive(Clone)]
/// pub struct Icon {
///     internal: HashMap<Key, DynamicImage>
/// }
/// 
/// impl Decode for Icon {
///     type Key = Key;
/// 
///     fn read<R: Read>(r: R) -> io::Result<Self> {
///         // Some decoding in here . . .
///     }
/// 
///     fn len(&self) -> usize {
///         self.internal.len()
///     }
/// 
///     fn contains_key(key: &Self::Key) -> bool {
///         self.internal.contains_key(key)
///     }
/// 
///     fn get(&self, key: &Self::Key) -> Option<&Image> {
///         self.internal.get(key)
///     }
/// 
///     fn entries(&self) -> Iter<(Self::Key, Image)> {
///         let output = Vec::with_capacity(self.len());
/// 
///         for entry in self.internal {
///             output.push(entry);
///         }
/// 
///         output.iter()
///     }
/// }
/// ```
pub trait Decode<'a>: Sized {
    type Key: 'a + AsSize + Send + Sync;
    type Entries: Iterator<Item = (&'a Self::Key, &'a Image)>;

    /// Parses and loads an icon into memmory.
    fn read<R: Read + Seek>(r: R) -> Result<DecodingError, Self>;

    /// Returns the number of _entries_ contained in the icon.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let len = icon.len();
    /// ```
    fn len(&self) -> usize;

    /// Returns `true` if the icon includes an entry associated with `key`.
    /// Otherwise returns `false`.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// if icon.contains_key(&Key(32)) {
    ///     // Do this . . .
    /// } else {
    ///     // Do that . . .
    /// }
    /// ```
    fn contains_key(&self, key: &Self::Key) -> bool;
    
    /// Returns `Some(entry)` if the icon includes an entry associated with `key`.
    /// Otherwise returns `None`.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// if let Some(entry) = icon.entry(&Key(32)) {
    ///     // Process the entry . . .
    /// }
    /// ```
    fn get(&self, key: &Self::Key) -> Option<&Image>;

    /// Returns an iterator over the entries of the icon.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// for (key, image) in icon.entries() {
    ///     // Do something . . .
    /// }
    /// ```
    fn entries(&'a self) -> Self::Entries;
}

#[inline]
/// Converts _PNG_-encoded buffers to _raster graphics_.
pub fn png<R: Read + Seek>(read: &mut R) -> io::Result<DynamicImage> {
    load_raster(read, ImageFormat::PNG)
}

#[inline]
/// Converts _BMP_-encoded buffers to _raster graphics_.
pub fn bmp<R: Read + Seek>(read: &mut R) -> io::Result<DynamicImage> {
    load_raster(read, ImageFormat::BMP)
}

#[inline]
/// Converts _UTF8_-encoded _SVG_ strings to _vector graphics_.
pub fn svg<R: Read + Seek>(read: &mut R) -> io::Result<Tree> {
    load_vector(read)
}