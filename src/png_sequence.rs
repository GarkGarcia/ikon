extern crate tar;
extern crate png_encode_mini;

use crate::{Icon, SourceImage, Size, Result, Error};
use std::io::Write;
use nsvg::image::RgbaImage;

pub struct PngSequence<W: Write> {
    tar_builder: tar::Builder<W>
}

impl<W: Write> Icon<W> for PngSequence<W> {
    fn new(w: W) -> Self {
        PngSequence { tar_builder: tar::Builder::new(w) }
    }

    fn add_icon<F: FnMut(&SourceImage, Size) -> Result<RgbaImage>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        size: Size
    ) -> Result<()> {
        let icon = filter(source, size)?;
        let size = icon.width();
    
        // Encode the pixel data as PNG and store it in a Vec<u8>
        let mut data = Vec::with_capacity(icon.len());
        if let Err(err) = png_encode_mini::write_rgba_from_u8(&mut data, &icon.into_raw(), size, size) {
            return Err(Error::Io(err));
        }
    
        let file_name = format!("/{}.png", size);
    
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_cksum();
    
        if let Err(err) = self.tar_builder.append_data::<String, &[u8]>(&mut header, file_name, data.as_ref()) {
            Err(Error::Io(err))
        } else {
            Ok(())
        }
    }
}
