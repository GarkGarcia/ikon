//! Traits, types and functions to assist in dencoding commonly used 
//! _icon formats_.

use crate::{load_raster, load_vector, Icon, Image};
pub use error::DecodingError;
use std::{io::{self, Read, Seek}};
use image::{ImageFormat, DynamicImage};
use resvg::usvg::Tree;

mod error;

/// The `Decode` trait represents a generic _icon family_ decoder, providing 
/// methods for generating icons from byte streams, as well as functionality 
/// querying and inspecting _icon families_.
/// 
/// # Example
/// 
/// In this example we'll create a very simple `Decode` implementor whose
/// icons are _positive integers_. First of all, we'll need a `Icon` type:
/// 
/// ```rust
/// #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// pub struct Icon(pub u16);
/// 
/// impl Icon for ikon::Icon {
///     fn size(&self) -> u32 {
///         if self.0 == 0 {
///             256
///         } else {
///             *self.0
///         }
///     }
/// }
/// ```
/// 
/// Note that `Icon(0)` represents `Icon(256)`. We can then implement our 
/// `IconFamily` type.
/// 
/// ```rust
/// #[derive(Clone)]
/// pub struct IconFamily {
///     internal: HashMap<Icon, DynamicImage>
/// }
/// 
/// impl Decode for IconFamily {
///     type Icon = Icon;
/// 
///     fn read<R: Read>(r: R) -> io::Result<Self> {
///         // Some decoding in here . . .
///     }
/// 
///     fn len(&self) -> usize {
///         self.internal.len()
///     }
/// 
///     fn contains_icon(icon: &Self::Icon) -> bool {
///         self.internal.contains_entry(icon)
///     }
/// 
///     fn get(&self, icon: &Self::Icon) -> Option<&Image> {
///         self.internal.get(icon)
///     }
/// 
///     fn icons(&self) -> Iter<(Self::Icon, Image)> {
///         let output = Vec::with_capacity(self.len());
/// 
///         for icon in self.internal {
///             output.push(icon);
///         }
/// 
///         output.iter()
///     }
/// }
/// ```
pub trait Decode<'a>: Sized {
    type Icon: 'a + Icon + Send + Sync;
    type Icons: Iterator<Item = (&'a Self::Icon, &'a Image)>;

    /// Parses and loads an icon into memmory.
    fn read<R: Read + Seek>(r: R) -> Result<Self, DecodingError>;

    /// Returns the number of _icons_ contained in the icon.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let len = icon.len();
    /// ```
    fn len(&self) -> usize;

    /// Returns `true` if the icon includes an icon associated with `icon`.
    /// Otherwise returns `false`.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// if icon.contains_icon(&Icon(32)) {
    ///     // Do this . . .
    /// } else {
    ///     // Do that . . .
    /// }
    /// ```
    fn contains_icon(&self, icon: &Self::Icon) -> bool;
    
    /// Returns `Some(icon)` if the icon includes an icon associated with `icon`.
    /// Otherwise returns `None`.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// if let Some(icon) = icon.icon(&Icon(32)) {
    ///     // Process the icon . . .
    /// }
    /// ```
    fn get(&self, icon: &Self::Icon) -> Option<&Image>;

    /// Returns an iterator over the icons of the icon.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// for (icon, image) in icon.icons() {
    ///     // Do something . . .
    /// }
    /// ```
    fn icons(&'a self) -> Self::Icons;
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

