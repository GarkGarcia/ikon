extern crate image;
extern crate tar;

use crate::{resample, Icon, FileLabel, SourceImage, Error, STD_CAPACITY};
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

impl Icon<FileLabel> for PngSequence {
    fn new() -> Self {
        PngSequence {
            entries: HashMap::with_capacity(STD_CAPACITY),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        filter: F,
        source: &SourceImage,
        entry: FileLabel,
    ) -> Result<(), Error<FileLabel>> {
        if entry.0 < MIN_PNG_SIZE {
            return Err(Error::InvalidDimensions(entry.0));
        }

        if self.entries.contains_key(&entry.1) {
            return Err(Error::AlreadyIncluded(entry));
        }

        let icon = resample::safe_filter(filter, source, entry.0)?;
        let data = icon.to_rgba().into_raw();
        
        // Encode the pixel data as PNG and store it in a Vec<u8>
        let mut image = Vec::with_capacity(data.len());
        let encoder = PNGEncoder::new(&mut image);
        encoder.encode(&data, entry.0, entry.0, ColorType::RGBA(8))?;

        match self.entries.insert(entry.1, image) {
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
