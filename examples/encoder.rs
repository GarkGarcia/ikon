extern crate ikon;

use std::{collections::hash_map::{HashMap, Entry}, fmt::Debug};
pub use ikon::{AsSize, Image, EncodingError, DynamicImage};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Key(pub u16);

#[derive(Clone)]
pub struct Icon {
    internal: HashMap<u16, Image>
}

impl ikon::Encoder for Icon {
    type Key = Key;

    fn with_capacity(capacity: usize) -> Self {
        Self { internal: HashMap::with_capacity(capacity) }
    }

    fn add_entry<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &Image,
        key: Self::Key,
    ) -> Result<(), EncodingError<Self::Key>> {
        let size = key.as_size();

        if let Entry::Vacant(entry) = self.internal.entry(size) {
            entry.insert(source.rasterize(filter, size);
            Ok(())
        } else {
            Err(EncodingError::AlreadyIncluded(key))
        }
    }
}