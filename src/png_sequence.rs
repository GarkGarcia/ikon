extern crate tar;
extern crate image;

use crate::{Icon, SourceImage, NamedEntry, Error};
use std::{io::{self, Write}, collections::HashMap};
use image::{png::PNGEncoder, DynamicImage, GenericImageView, ImageError, ColorType};

const MIN_PNG_SIZE: u32 = 1;
const STD_CAPACITY: usize = 7;

/// A collection of images stored in a single `.tar` file.
#[derive(Clone, Debug)]
pub struct PngSequence {
    images: HashMap<NamedEntry, Vec<u8>>
}

impl Icon<NamedEntry> for PngSequence {
    fn new() -> Self {
        PngSequence { images: HashMap::with_capacity(STD_CAPACITY) }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        entry: NamedEntry
    ) -> Result<(), Error<NamedEntry>> {
        if entry.0 < MIN_PNG_SIZE {
            return Err(Error::InvalidSize(entry.0));
        }

        if self.images.contains_key(&entry) {
            return Err(Error::AlreadyIncluded(entry));
        }

        let icon = filter(source, entry.0);
        if icon.width() != entry.0 || icon.height() != entry.0 {
            return Err(Error::Image(ImageError::DimensionError));
        }

        let data = icon.to_rgba().into_raw();
    
        // Encode the pixel data as PNG and store it in a Vec<u8>
        let mut image = Vec::with_capacity(data.len());
        let encoder = PNGEncoder::new(&mut image);
        encoder.encode(&data, entry.0, entry.0, ColorType::RGBA(8))?;

        match self.images.insert(entry, image) {
            Some(img) => panic!("Sanity test failed: {:?} is already included.", img),
            None    => Ok(())
        }
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let mut tar_builder = tar::Builder::new(w);

        for (entry, image) in &self.images {
            let mut header = tar::Header::new_gnu();
            header.set_size(image.len() as u64);
            header.set_cksum();

            tar_builder.append_data::<_, &[u8]>(
                &mut header,
                entry.1.clone(),
                image.as_ref()
            )?;
        }

        Ok(())
    }
}
