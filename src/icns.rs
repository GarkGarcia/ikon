extern crate icns;

use crate::{Entry, Icon, SourceImage, Error, STD_CAPACITY};
use image::{DynamicImage, GenericImageView};
use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Write},
};

/// A collection of entries stored in a single `.icns` file.
pub struct Icns {
    icon_family: icns::IconFamily,
    entries: Vec<IcnsEntry>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IcnsEntry {E16, E32, E64, E128, E256, E512, E1024}

impl Icon<IcnsEntry> for Icns {
    fn new() -> Self {
        Icns {
            icon_family: icns::IconFamily::new(),
            entries: Vec::with_capacity(STD_CAPACITY),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        entry: IcnsEntry
    ) -> Result<(), Error<IcnsEntry>> {
        let size = entry.size();
        let icon = filter(source, size);
        let data = icon.to_rgba().into_vec();

        if self.entries.contains(&entry) {
            return Err(Error::AlreadyIncluded(entry));
        }

        // The Image::from_data method only fails when the specified
        // image dimensions do not fit the buffer length
        let image = icns::Image::from_data(icns::PixelFormat::RGBA, size, size, data)
            .map_err(|_| Error::MismatchedDimensions(size, icon.dimensions()))?;

        // The IconFamily::add_icon method only fails when the
        // specified image dimensions are not supported by ICNS
        self.icon_family
            .add_icon(&image)
            .expect("This should not fail!");

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
            entries: self.entries.clone(),
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

impl Entry for IcnsEntry {
    fn size(&self) -> u32 {
        match self {
            IcnsEntry::E16   => 16,
            IcnsEntry::E32   => 32,
            IcnsEntry::E64   => 64,
            IcnsEntry::E128  => 128,
            IcnsEntry::E256  => 256,
            IcnsEntry::E512  => 512,
            IcnsEntry::E1024 => 1024
        }
    }
}
