extern crate ico;

use crate::{resample, Entry, Error, Icon, SourceImage, STD_CAPACITY};
use image::DynamicImage;
use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Write},
    result,
    num::NonZeroU8
};

/// A collection of entries stored in a single `.ico` file.
#[derive(Clone)]
pub struct Ico {
    icon_dir: ico::IconDir,
    entries: Vec<NonZeroU8>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct IcoEntry(NonZeroU8);

impl Icon<IcoEntry> for Ico {
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
        entry: IcoEntry,
    ) -> Result<(), Error<IcoEntry>> {
        if self.entries.contains(&entry.0) {
            return Err(Error::AlreadyIncluded(entry));
        }

        let size = entry.size();
        let icon = resample::safe_filter(filter, source, size)?;
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

impl IcoEntry {
    pub fn new(n: u8) -> Option<Self> {
        let raw = NonZeroU8::new(n)?;

        Some(IcoEntry(raw))        
    }
}

impl Entry for IcoEntry {
    fn size(&self) -> u32 {
        self.0.get() as u32
    }
}
