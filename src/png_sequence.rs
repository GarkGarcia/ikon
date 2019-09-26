//! Structs for encoding _png sequences_ (series of `.png` files indexed by _path_).

extern crate image;
extern crate tar;

use crate::{resample, AsSize, Error, Icon, SourceImage, STD_CAPACITY};
use image::{png::PNGEncoder, ColorType, DynamicImage};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    num::NonZeroU32
};

/// A collection of entries stored in a single `.tar` file.
#[derive(Clone, Debug)]
pub struct PngSequence {
    entries: HashMap<PathBuf, Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PngKey {
    size: NonZeroU32,
    path: PathBuf
}

impl Icon for PngSequence {
    type Key = PngKey;

    fn new() -> Self {
        PngSequence {
            entries: HashMap::with_capacity(STD_CAPACITY),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        key: Self::Key,
    ) -> Result<(), Error<Self::Key>> {
        let size = key.as_size();

        if self.entries.contains_key(&key.path) {
            return Err(Error::AlreadyIncluded(key));
        }

        let icon = resample::safe_filter(filter, source, size)?;
        let data = icon.to_rgba().into_raw();
        // Encode the pixel data as PNG and store it in a Vec<u8>
        let mut image = Vec::with_capacity(data.len());
        let encoder = PNGEncoder::new(&mut image);
        encoder.encode(&data, size, size, ColorType::RGBA(8))?;

        match self.entries.insert(key.path, image) {
            Some(img) => panic!("Sanity test failed: {:?} is already included.", img),
            None => Ok(()),
        }
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let mut tar_builder = tar::Builder::new(w);

        for (path, image) in &self.entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(image.len() as u64);
            header.set_cksum();

            tar_builder.append_data::<_, &[u8]>(&mut header, path.clone(), image.as_ref())?;
        }

        Ok(())
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

impl PngKey {
    pub fn from<P: Into<PathBuf>>(size: u32, path: P) -> Option<Self> {
        Some(PngKey { size: NonZeroU32::new(size)?, path: path.into() })
    }
}

impl AsSize for PngKey {
    fn as_size(&self) -> u32 {
        self.size.get()
    }
}
