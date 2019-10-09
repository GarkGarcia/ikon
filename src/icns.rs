//! Structs for encoding `.icns` files.

extern crate icns;

use crate::{Icon, AsSize, SourceImage, Error};
use image::{DynamicImage, GenericImageView};
use std::{
    str::FromStr,
    convert::TryFrom,
    fmt::{self, Debug, Formatter},
    io::{self, Write},
};

/// An ecoder for the `.icns` file format.
pub struct Icns {
    icon_family: icns::IconFamily,
    keys: Vec<u32>,
}

/// The _key-type_ for `Icns`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Rgba16,
    Rgba32,
    Rgba64,
    Rgba128,
    Rgba256,
    Rgba512,
    Rgba1024
}

impl Icon for Icns {
    type Key = Key;
    type WriteOptions = ();

    fn with_capacity(capacity: usize) -> Self {
        Icns {
            icon_family: icns::IconFamily { elements: Vec::with_capacity(capacity) },
            keys: Vec::with_capacity(capacity),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        key: Self::Key
    ) -> Result<(), Error<Self::Key>> {
        let size = key.as_size();
        let icon = filter(source, size)?;
        let data = icon.to_rgba().into_vec();

        if self.keys.contains(&size) {
            return Err(Error::AlreadyIncluded(key));
        }

        // The Image::from_data method only fails when the specified
        // image dimensions do not fit the buffer length
        let image = icns::Image::from_data(icns::PixelFormat::RGBA, size, size, data)
            .map_err(|_| Error::MismatchedDimensions(size, icon.dimensions()))?;

        // The IconFamily::add_icon method only fails when the
        // specified image dimensions are not supported by ICNS
        self.icon_family
            .add_icon(&image)
            .expect("The image dimensions should be supported by ICNS");

        Ok(())
    }

    fn write<W: Write>(&mut self, w: &mut W, _: &()) -> io::Result<()> {
        self.icon_family.write(w)
    }
}

impl Clone for Icns {
    fn clone(&self) -> Self {
        let mut icon_family = icns::IconFamily {
            elements: Vec::with_capacity(self.icon_family.elements.len()),
        };

        for element in &self.icon_family.elements {
            let clone = icns::IconElement::new(element.ostype, element.data.clone());

            icon_family.elements.push(clone);
        }

        Icns {
            icon_family,
            keys: self.keys.clone(),
        }
    }
}

macro_rules! element {
    ($elm:expr) => {
        format!(
            "IconElement {{ ostype: {:?}, data: {:?} }}",
            $elm.ostype, $elm.data
        )
    };
}

impl Debug for Icns {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let entries_strs: Vec<String> = self
            .icon_family
            .elements
            .iter()
            .map(|element| element!(element))
            .collect();

        let icon_dir = format!(
            "icns::IconFamily {{ elements: [{}] }}",
            entries_strs.join(", ")
        );

        write!(f, "icon_baker::Icns {{ icon_family: {} }} ", icon_dir)
    }
}

impl AsSize for Key {
    fn as_size(&self) -> u32 {
        match self {
            Self::Rgba1024 => 1024,
            Self::Rgba512 => 512,
            Self::Rgba256 => 256,
            Self::Rgba128 => 128,
            Self::Rgba64 => 64,
            Self::Rgba32 => 32,
            Self::Rgba16 => 16
        }
    }
}

impl TryFrom<u32> for Key {
    type Error = io::Error;

    fn try_from(size: u32) -> io::Result<Self> {
        match size {
            1024 => Ok(Self::Rgba1024),
            512 => Ok(Self::Rgba512),
            256 => Ok(Self::Rgba256),
            128 => Ok(Self::Rgba128),
            64 => Ok(Self::Rgba64),
            32 => Ok(Self::Rgba32),
            16 => Ok(Self::Rgba16),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput))
        }
    }
}

impl FromStr for Key {
    type Err = io::Error;

    fn from_str(s: &str) -> io::Result<Self> {
        match s {
            "1024" => Ok(Self::Rgba1024),
            "512" => Ok(Self::Rgba512),
            "256" => Ok(Self::Rgba256),
            "128" => Ok(Self::Rgba128),
            "64" => Ok(Self::Rgba64),
            "32" => Ok(Self::Rgba32),
            "16" => Ok(Self::Rgba16),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput))
        }
    }
}
