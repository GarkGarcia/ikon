//! Traits, types and functions to assist in decoding commonly used 
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
/// Note that `Icon(0)` represents `Icon(256)`. We can then implement our 
/// `IconFamily` type.
/// 
/// ```rust
/// use std::{io::{self, Read}, collections::HashMap, hash::Hash, slice::Iter};
/// use ikon::{decode::{Decode, DecodingError}, Image};
///
/// #[derive(Clone)]
/// pub struct IconFamily<Icon: ikon::Icon + Send + Sync + Eq + Hash> {
///     internal: HashMap<Icon, Image>
/// }
/// 
/// impl<'a, Icon> Decode<'a> for IconFamily<Icon> 
///     where Icon: 'a + ikon::Icon + Send + Sync + Eq + Hash
/// {
///     type Icon = Icon;
///     type Icons = Iter<'a, (Self::Icon, Image)>;
/// 
///     fn read<R: Read>(r: R) -> Result<Self, DecodingError> {
///         unimplemented!("Some decoding in here . . .");
///     }
/// 
///     fn len(&self) -> usize {
///         self.internal.len()
///     }
/// 
///     fn contains_icon(&self, icon: &Self::Icon) -> bool {
///         self.internal.contains_key(icon)
///     }
/// 
///     fn get(&self, icon: &Self::Icon) -> Option<&Image> {
///         self.internal.get(icon)
///     }
/// }
/// ```
pub trait Decode<'a>: Sized {
    /// The type of icon of the icon family.
    type Icon: 'a + Icon + Send + Sync;

    /// The return type of `Decode::icons`.
    type Icons: Iterator<Item = &'a (Self::Icon, Image)>;

    /// Parses and loads an icon family into memmory.
    fn read<R: Read + Seek>(r: R) -> Result<Self, DecodingError>;

    /// Returns the number of _icons_ contained in the icon family.
    fn len(&self) -> usize;

    /// Returns `true` if the icon family contains `icon`.
    /// Otherwise returns `false`.
    fn contains_icon(&self, icon: &Self::Icon) -> bool;
    
    /// Returns `Some(icon)` if the icon family contains `icon`.
    fn get(&self, icon: &Self::Icon) -> Option<&Image>;
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

