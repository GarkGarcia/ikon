extern crate icns;

use crate::{Icon, SourceImage, Size, Result, Error};
use std::io::{Write, BufWriter};
use nsvg::image::RgbaImage;

pub struct Icns<W: Write> {
    icon_family: icns::IconFamily,
    buf_writer: BufWriter<W>
}

impl <W: Write> Icns<W> {
    fn insert_icon<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
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
        let mut buf: Vec<u8> = Vec::new();

        match self.icon_family.write::<&mut [u8]>(buf.as_mut()) {
            Ok(_) => self.buf_writer.write_all(buf.as_mut()).map_err(|err| Error::Io(err)),
            Err(err) => Err(Error::Io(err))
        }
    }
}

impl<W: Write> Icon<W> for Icns<W> {
    fn new(w: W) -> Self {
        Icns { icon_family: icns::IconFamily::new(), buf_writer: BufWriter::new(w) }
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