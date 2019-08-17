extern crate ico;

use crate::{Icon, SourceImage, Size, Result, Error};
use std::{result, io::{self, Write}, fmt::{self, Debug, Formatter}};
use nsvg::image::RgbaImage;

#[derive(Clone)]
pub struct Ico {
    icon_dir: ico::IconDir
}

impl Icon for Ico {
    fn new() -> Self {
        Ico { icon_dir: ico::IconDir::new(ico::ResourceType::Icon) }
    }

    fn add_entry<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()> {
        let icon = filter(source, size)?;
        let size = icon.width();
        let data = ico::IconImage::from_rgba_data(size, size, icon.clone().into_vec());

        match ico::IconDirEntry::encode(&data) {
            Ok(entry) => self.icon_dir.add_entry(entry),
            Err(err)  => return Err(Error::Io(err))
        }

        Ok(())
    }

    fn add_entries<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>, I: IntoIterator<Item = Size>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        sizes: I
    ) -> Result<()> {
        for size in sizes.into_iter() {
            self.add_entry(|src, size| filter(src, size), source, size)?;
        }

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