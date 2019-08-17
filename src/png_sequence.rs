extern crate tar;
extern crate png_encode_mini;

use crate::{Icon, SourceImage, Size, Result, Error};
use std::{io::Write, collections::HashMap};
use nsvg::image::RgbaImage;

#[derive(Clone, Debug)]
pub struct PngSequence {
    pngs: HashMap<Size, Vec<u8>>
}

impl Icon for PngSequence {
    fn new() -> Self {
        PngSequence { pngs: HashMap::with_capacity(7) }
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

        if let Some(_) = self.pngs.insert(size, data) {
            unimplemented!()
        } else {
            Ok(())
        }
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

    fn write<W: Write>(&mut self, w: &mut W) -> Result<()> {
        let mut tar_builder = tar::Builder::new(w);

        for (size, data) in &self.pngs {
            let path = format!("./{}.png", size);

            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_cksum();

            tar_builder
                .append_data::<String, &[u8]>(&mut header, path, data.as_ref())
                .map_err(|err| Error::Io(err))?;
        }

        Ok(())
    }
}
