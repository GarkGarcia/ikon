//! Structs for encoding `.icns` files.

extern crate icns;

use crate::{SizeKey, Icon, SourceImage, Error, STD_CAPACITY};
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

impl Icon<SizeKey> for Icns {
    fn new() -> Self {
        Icns {
            icon_family: icns::IconFamily::new(),
            keys: Vec::with_capacity(STD_CAPACITY),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        key: SizeKey
    ) -> Result<(), Error<SizeKey>> {
        let icon = filter(source, key.0);
        let data = icon.to_rgba().into_vec();

        if self.keys.contains(&key.0) {
            return Err(Error::AlreadyIncluded(key));
        }

        // The Image::from_data method only fails when the specified
        // image dimensions do not fit the buffer length
        let image = icns::Image::from_data(icns::PixelFormat::RGBA, key.0, key.0, data)
            .map_err(|_| Error::MismatchedDimensions(key.0, icon.dimensions()))?;

        // The IconFamily::add_icon method only fails when the
        // specified image dimensions are not supported by ICNS
        self.icon_family
            .add_icon(&image)
            .map_err(|_| Error::InvalidDimensions(key.0))
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
