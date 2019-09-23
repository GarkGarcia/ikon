//! Structs for encoding _png sequences_ (series of `.png` files indexed by _path_).

extern crate image;
extern crate tar;

use crate::{resample, Icon, PathKey, SourceImage, Error, STD_CAPACITY};
use image::{png::PNGEncoder, ColorType, DynamicImage};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

const MIN_PNG_SIZE: u32 = 1;

/// A collection of entries stored in a single `.tar` file.
#[derive(Clone, Debug)]
pub struct PngSequence {
    entries: HashMap<PathBuf, Vec<u8>>,
}

impl PngSequence {
    #[inline]
    pub(crate) fn write_to_tar<W: Write>(
        &self,
        builder: &mut tar::Builder<W>
    ) -> io::Result<()> {
        for (path, image) in &self.entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(image.len() as u64);
            header.set_cksum();

            builder.append_data::<_, &[u8]>(&mut header, path.clone(), image.as_ref())?;
        }

        Ok(())
    }
}

impl Icon<PathKey> for PngSequence {
    fn new() -> Self {
        PngSequence {
            entries: HashMap::with_capacity(STD_CAPACITY),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        filter: F,
        source: &SourceImage,
        key: PathKey,
    ) -> Result<(), Error<PathKey>> {
        if key.0 < MIN_PNG_SIZE {
            return Err(Error::InvalidDimensions(key.0));
        }

        if self.entries.contains_key(&key.1) {
            return Err(Error::AlreadyIncluded(key));
        }

        let icon = resample::safe_filter(filter, source, key.0)?;
        let data = icon.to_rgba().into_raw();
        
        // Encode the pixel data as PNG and store it in a Vec<u8>
        let mut image = Vec::with_capacity(data.len());
        let encoder = PNGEncoder::new(&mut image);
        encoder.encode(&data, key.0, key.0, ColorType::RGBA(8))?;

        match self.entries.insert(key.1, image) {
            Some(img) => panic!("Sanity test failed: {:?} is already included.", img),
            None => Ok(()),
        }
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let mut tar_builder = tar::Builder::new(w);
        self.write_to_tar(&mut tar_builder)
    }

    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<()> {
        if path.as_ref().is_file() {
            let mut file = File::create(path.as_ref())?;
            self.write(&mut file)
        } else {
            for (path, image) in &self.entries {
                let mut file = File::create(path.clone())?;
                file.write_all(image.as_ref())?;
            }

            Ok(())
        }
    }
}
