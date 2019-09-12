extern crate ico;

use crate::{Icon, SourceImage, Entry, Error};
use std::{result, io::{self, Write}, fmt::{self, Debug, Formatter}};
use image::{DynamicImage, ImageError, GenericImageView};

const MIN_ICO_SIZE: u32 = 1;
const MAX_ICO_SIZE: u32 = 256;

/// A collection of entries stored in a single `.ico` file.
#[derive(Clone)]
pub struct Ico {
    icon_dir: ico::IconDir
}

impl Icon<Entry> for Ico {
    fn new() -> Self {
        Ico { icon_dir: ico::IconDir::new(ico::ResourceType::Icon) }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> Result<DynamicImage, Error<Entry>>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        entry: Entry
    ) -> Result<(), Error<Entry>> {
        if entry.0 < MIN_ICO_SIZE || entry.0 > MAX_ICO_SIZE {
            return Err(Error::InvalidSize(entry.0));
        }

        let icon = filter(source, entry.0)?;
        if icon.width() != entry.0 || icon.height() != entry.0 {
            return Err(Error::Image(ImageError::DimensionError));
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

        let icon_dir= format!(
            "ico::IconDir {{ restype: ico::ResourceType::Icon, entries: [{:?}] }}",
            entries_str
        );

        write!(f, "icon_baker::Ico {{ icon_dir: {} }} ", icon_dir)
    }
}