extern crate tar;
extern crate png_encode_mini;

use crate::{Icon, SourceImage, Size, Result, Error};
use std::io::{self, Write};
use nsvg::image::RgbaImage;

pub struct PngSequence<W: Write> {
    tar_builder: tar::Builder<W>
}

impl<W: Write> Icon<W> for PngSequence<W> {
    fn new(w: W) -> Self {
        PngSequence { tar_builder: tar::Builder::new(w) }
    }

    fn add_entry<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()> {
        let icon = filter(source, size)?;
        let size = icon.width();
    
        // Encode the pixel data as PNG and store it in a Vec<u8>
        let mut data = Vec::with_capacity(icon.len());
        if let Err(err) = png_encode_mini::write_rgba_from_u8(
            &mut data,
            &icon.into_raw(),
            size,
            size
        ) {
            return Err(Error::Io(err));
        }
    
        let file_name = format!("/{}.png", size);
    
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_cksum();

        self.tar_builder
            .append_data::<String, &[u8]>(&mut header, file_name, data.as_ref())
            .map_err(|err| Error::Io(err))
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

    fn len(&self) -> usize {
        unimplemented!()
    }

    fn into_inner(self) -> io::Result<W> {
        self.tar_builder.into_inner()
    }
}

impl<W: Write> AsRef<W> for PngSequence<W> {
    fn as_ref(&self) -> &W {
        self.tar_builder.get_ref()
    }
}
