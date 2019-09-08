extern crate tar;
extern crate image;

use crate::{Icon, SourceImage, Entry, Result, Error};
use std::{io::{self, Write}, collections::{HashMap, BTreeSet}};
use image::{png::PNGEncoder, DynamicImage, GenericImageView, ImageError, ColorType};

const MIN_PNG_SIZE: u32 = 1;
const STD_CAPACITY: usize = 7;

/// A collection of images stored in a single `.tar` file.
#[derive(Clone, Debug)]
pub struct PngSequence {
    images: HashMap<u32, BTreeSet<Vec<u8>>>
}

impl Icon<Entry> for PngSequence {
    fn new() -> Self {
        PngSequence { images: HashMap::with_capacity(STD_CAPACITY) }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> Result<DynamicImage>>(
        &mut self,
        mut filter: F,
        source: &SourceImage,
        entry: Entry
    ) -> Result<()> {
        if entry.0 < MIN_PNG_SIZE {
            return Err(Error::InvalidSize(entry.0));
        }

        let icon = filter(source, entry.0)?;
        if icon.width() != entry.0 || icon.height() != entry.0 {
            return Err(Error::Image(ImageError::DimensionError));
        }

        let data = icon.to_rgba().into_raw();
    
        // Encode the pixel data as PNG and store it in a Vec<u8>
        let mut image = Vec::with_capacity(data.len());
        let encoder = PNGEncoder::new(&mut image);
        encoder.encode(&data, entry.0, entry.0, ColorType::RGBA(8))?;

        self.images.entry(entry.0).or_default().insert(image);
        Ok(())
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let mut tar_builder = tar::Builder::new(w);

        macro_rules! append {
            ($image:expr, $path:expr) => {
                let mut header = tar::Header::new_gnu();
                header.set_size($image.len() as u64);
                header.set_cksum();
    
                tar_builder
                    .append_data::<String, &[u8]>(&mut header, $path, $image.as_ref())?;
            };
        }

         for (size, images) in &self.images {
            if images.len() == 1 {
                let path = format!("./{}/icon.png", size);
                for image in images { append!(image, path); break; }
            } else {
                let mut c = 0;

                for image in images {
                    let path = format!("./{}/icon@{}.png", size, c);
                    append!(image, path);

                    c += 1;
                }
            }
        }

        Ok(())
    }
}
