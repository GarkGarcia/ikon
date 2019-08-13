extern crate ico;

use crate::{Icon, SourceImage, Size, Result, Error};
use std::io::{Write, BufWriter};
use nsvg::image::RgbaImage;

pub struct Ico<W: Write> {
    icon_dir: ico::IconDir,
    buf_writer: BufWriter<W>
}

impl<W: Write> Ico<W> {
    fn insert_icon<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
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
            Err(err) => return Err(Error::Io(err))
        }

        Ok(())
    }

    fn write(&mut self) -> Result<()> {
        let mut buf: Vec<u8> = Vec::new();

        match self.icon_dir.write::<&mut [u8]>(buf.as_mut()) {
            Ok(_) => self.buf_writer.write_all(buf.as_mut()).map_err(|err| Error::Io(err)),
            Err(err) => Err(Error::Io(err))
        }
    }
}

impl<W: Write> Icon<W> for Ico<W> {
    fn new(w: W) -> Self {
        Ico { icon_dir: ico::IconDir::new(ico::ResourceType::Icon), buf_writer: BufWriter::new(w) }
    }

    fn add_icon<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()> {
            self.insert_icon(filter, source, size)?;
            self.write()
    }
}