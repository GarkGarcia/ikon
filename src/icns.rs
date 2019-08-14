extern crate icns;

use crate::{Icon, SourceImage, Size, Result, Error};
use std::io::{self, Write};
use nsvg::image::RgbaImage;

pub struct Icns<W: Write> {
    icon_family: icns::IconFamily,
    writer: W
}

impl <W: Write> Icns<W> {
    fn insert_entry<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()> {
        let icon = filter(source, size)?;

        match icns::Image::from_data(icns::PixelFormat::RGBA, size, size, icon.into_vec()) {
            Ok(icon) => self.icon_family.add_icon(&icon).map_err(|err| Error::Io(err)),
            Err(err) => Err(Error::Io(err))
        }
    }

    fn write(&mut self) -> Result<()> {
        let mut buf: Vec<u8> = Vec::with_capacity(self.icon_family.total_length() as usize);

        match self.icon_family.write::<&mut [u8]>(buf.as_mut()) {
            Ok(_)    => self.writer.write_all(buf.as_mut()).map_err(|err| Error::Io(err)),
            Err(err) => Err(Error::Io(err))
        }
    }
}

impl<W: Write> Icon<W> for Icns<W> {
    fn new(w: W) -> Self {
        Icns { icon_family: icns::IconFamily::new(), writer: w }
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

impl<W: Write> AsRef<W> for Icns<W> {
    fn as_ref(&self) -> &W {
        &self.writer
    }
}