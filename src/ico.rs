extern crate ico;

use crate::{Icon, SourceImage, Size, Result, Error};
use std::io::{self, Write};
use nsvg::image::RgbaImage;

pub struct Ico<W: Write> {
    icon_dir: ico::IconDir,
    writer: W,
    length: usize
}

impl<W: Write> Ico<W> {
    fn insert_entry<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
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

    fn write(&mut self) -> Result<()> {
        let mut buf: Vec<u8> = Vec::with_capacity(self.length);

        match self.icon_dir.write::<&mut [u8]>(buf.as_mut()) {
            Ok(_)    => {
                self.length = buf.len();
                self.writer.write_all(buf.as_mut()).map_err(|err| Error::Io(err))
            },
            Err(err) => Err(Error::Io(err))
        }
    }
}

impl<W: Write> Icon<W> for Ico<W> {
    fn new(w: W) -> Self {
        Ico { icon_dir: ico::IconDir::new(ico::ResourceType::Icon), writer: w, length: 0 }
    }

    fn add_entry<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()> {
            self.insert_entry(filter, source, size)?;
            self.write()
    }

    fn add_entries<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>, I: IntoIterator<Item = Size>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        sizes: I
    ) -> Result<()> {
        for size in sizes.into_iter() {
            self.insert_entry(|src, size| filter(src, size), source, size)?;
        }

        self.write()
    }

    fn into_inner(self) -> io::Result<W> {
        Ok(self.writer)
    }
}

impl<W: Write> AsRef<W> for Ico<W> {
    fn as_ref(&self) -> &W {
        &self.writer
    }
}