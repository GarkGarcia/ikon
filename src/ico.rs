extern crate ico;

use crate::{Entry, Error, Icon, SourceImage, STD_CAPACITY};
use image::{DynamicImage, GenericImageView};
use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Write},
    result,
};

const MIN_ICO_SIZE: u32 = 1;
const MAX_ICO_SIZE: u32 = 256;

/// A collection of entries stored in a single `.ico` file.
#[derive(Clone)]
pub struct Ico {
    icon_dir: ico::IconDir,
    entries: Vec<u32>,
}

impl Icon<Entry> for Ico {
    fn new() -> Self {
        Ico {
            icon_dir: ico::IconDir::new(ico::ResourceType::Icon),
            entries: Vec::with_capacity(STD_CAPACITY),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        entry: Entry,
    ) -> Result<(), Error<Entry>> {
        if entry.0 < MIN_ICO_SIZE || entry.0 > MAX_ICO_SIZE {
            return Err(Error::InvalidDimensions(entry.0));
        }

        if self.entries.contains(&entry.0) {
            return Err(Error::AlreadyIncluded(entry));
        }

        let icon = filter(source, entry.0);
        let (icon_w, icon_h) = icon.dimensions();
        if icon_w != entry.0 || icon_h != entry.0 {
            return Err(Error::MismatchedDimensions(entry.0, (icon_w, icon_h)));
        }

        let size = icon.width();
        let data = icon.to_rgba().into_vec();
        let image = ico::IconImage::from_rgba_data(size, size, data);

        let entry = ico::IconDirEntry::encode(&image)?;
        self.icon_dir.add_entry(entry);

        Ok(())
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.icon_dir.write(w)
    }
}

impl Debug for Ico {
    fn fmt(&self, f: &mut Formatter) -> result::Result<(), fmt::Error> {
        let n_entries = self.icon_dir.entries().len();
        let mut entries_str = String::with_capacity(42 * n_entries);

        for _ in 0..n_entries {
            entries_str.push_str("ico::IconDirEntry {{ /* fields omitted */ }}, ");
        }

        let icon_dir = format!(
            "ico::IconDir {{ restype: ico::ResourceType::Icon, entries: [{:?}] }}",
            entries_str
        );

        write!(f, "icon_baker::Ico {{ icon_dir: {} }} ", icon_dir)
    }
}
