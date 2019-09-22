extern crate ico;

use crate::{resample, Size, Error, Icon, SourceImage, STD_CAPACITY};
use image::DynamicImage;
use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Write},
    result,
};

const MIN_ICO_SIZE: u32 = 1;
const MAX_ICO_SIZE: u32 = 256;

/// An ecoder for the `.ico` file format.
#[derive(Clone)]
pub struct Ico {
    icon_dir: ico::IconDir,
    entries: Vec<u32>,
}

impl Icon<Size> for Ico {
    fn new() -> Self {
        Ico {
            icon_dir: ico::IconDir::new(ico::ResourceType::Icon),
            entries: Vec::with_capacity(STD_CAPACITY),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        filter: F,
        source: &SourceImage,
        entry: Size,
    ) -> Result<(), Error<Size>> {
        if entry.0 < MIN_ICO_SIZE || entry.0 > MAX_ICO_SIZE {
            return Err(Error::InvalidDimensions(entry.0));
        }

        if self.entries.contains(&entry.0) {
            return Err(Error::AlreadyIncluded(entry));
        }

        let icon = resample::safe_filter(filter, source, entry.0)?;
        let data = icon.to_rgba().into_vec();
        let image = ico::IconImage::from_rgba_data(entry.0, entry.0, data);

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
