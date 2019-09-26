//! Structs for encoding `.icns` files.

extern crate icns;

use crate::{Icon, AsSize, SourceImage, Error, STD_CAPACITY};
use image::{DynamicImage, GenericImageView};
use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Write},
};

/// An ecoder for the `.icns` file format.
pub struct Icns {
    icon_family: icns::IconFamily,
    keys: Vec<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IcnsKey {
    RGBA16,
    RGBA32,
    RGBA64,
    RGBA128,
    RGBA256,
    RGBA512,
    RGBA1024
}

impl Icon for Icns {
    type Key = IcnsKey;

    fn new() -> Self {
        Icns {
            icon_family: icns::IconFamily::new(),
            keys: Vec::with_capacity(STD_CAPACITY),
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

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
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

impl IcnsKey {
    pub fn from(size: u32) -> Option<Self> {
        match size {
            1024 => Some(Self::RGBA1024),
            512 => Some(Self::RGBA512),
            256 => Some(Self::RGBA256),
            128 => Some(Self::RGBA128),
            64 => Some(Self::RGBA64),
            32 => Some(Self::RGBA32),
            16 => Some(Self::RGBA16),
            _ => None
        }
    }
}

impl AsSize for IcnsKey {
    fn as_size(&self) -> u32 {
        match self {
            Self::RGBA1024 => 1024,
            Self::RGBA512 => 512,
            Self::RGBA256 => 256,
            Self::RGBA128 => 128,
            Self::RGBA64 => 64,
            Self::RGBA32 => 32,
            Self::RGBA16 => 16
        }
    }
}
