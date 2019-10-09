//! Structs for encoding `.ico` files.

extern crate ico;

use crate::{resample, AsSize, Error, Icon, SourceImage};
use image::DynamicImage;
use std::{
    convert::TryFrom,
    str::FromStr,
    fmt::{self, Debug, Formatter},
    io::{self, Write},
    result,
};

/// An ecoder for the `.ico` file format.
#[derive(Clone)]
pub struct Ico {
    icon_dir: ico::IconDir,
    keys: Vec<u32>,
}

/// The _key-type_ for `Ico`. Note that `Key(0)` represents
/// a _256x256_ entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key(pub u8);

impl Icon for Ico {
    type Key = Key;
    type WriteOptions = ();

    fn with_capacity(capacity: usize) -> Self {
        Ico {
            icon_dir: ico::IconDir::new(ico::ResourceType::Icon),
            keys: Vec::with_capacity(capacity),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        key: Self::Key,
    ) -> Result<(), Error<Self::Key>> {
        let size = key.as_size();

        if self.keys.contains(&size) {
            return Err(Error::AlreadyIncluded(key));
        }

        let icon = resample::safe_filter(filter, source, size)?;
        let data = icon.to_rgba().into_vec();
        let image = ico::IconImage::from_rgba_data(size, size, data);

        let entry = ico::IconDirEntry::encode(&image)?;
        self.icon_dir.add_entry(entry);

        Ok(())
    }

    fn write<W: Write>(&mut self, w: &mut W, _: &()) -> io::Result<()> {
        self.icon_dir.write(w)
    }
}

impl Debug for Ico {
    fn fmt(&self, f: &mut Formatter) -> result::Result<(), fmt::Error> {
        let n_entries = self.icon_dir.entries().len();
        let mut entries_str = String::with_capacity(42 * n_entries);

        for _ in 0..n_entries {
            entries_str.push_str("ico::IconDirEntry {{ /* fields omitted */ }}, ");
        }

        let icon_dir = format!(
            "ico::IconDir {{ restype: ico::ResourceType::Icon, entries: [{:?}] }}",
            entries_str
        );

        write!(f, "icon_baker::Ico {{ icon_dir: {} }} ", icon_dir)
    }
}

impl AsSize for Key {
    fn as_size(&self) -> u32 {
        if self.0 == 0 {
            256
        } else {
            self.0 as u32
        }
    }
}

impl TryFrom<u32> for Key {
    type Error = io::Error;

    fn try_from(val: u32) -> io::Result<Self> {
        match val {
            256 => Ok(Key(0)),
            0 => Err(io::Error::from(io::ErrorKind::InvalidInput)),
            n if n < 256 => Ok(Key(n as u8)),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput))
        }
    }
}

impl FromStr for Key {
    type Err = io::Error;

    fn from_str(s: &str) -> io::Result<Self> {
        match s {
            "256" => Ok(Key(0)),
            "0" => Err(io::Error::from(io::ErrorKind::InvalidInput)),
            _ => s.parse::<u8>()
                .map(Key)
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
        }
    }
}
