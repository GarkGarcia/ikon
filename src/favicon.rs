//! Structs for encoding _[favicons](https://en.wikipedia.org/wiki/Favicon)_.

extern crate image;
extern crate tar;

use crate::{
    png_sequence::PngSequence, Error, Icon, PathKey, SourceImage, STD_CAPACITY
};
use image::DynamicImage;
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    cmp::{Ord, Ordering}
};

macro_rules! path {
    ($format: expr, $($arg: expr)*) => {
        PathBuf::from(format!($format, $($arg)*))
    };
}

#[derive(Clone)]
/// A comprehencive _favicon_ builder.
pub struct Favicon {
    internal: PngSequence,
    keys: Vec<FaviconKey>
}

/// The _key type_ for `FavIcon`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord)]
pub enum FaviconKey {
    /// Variant for 
    /// _[Apple touch icons](https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/ConfiguringWebApplications/ConfiguringWebApplications.html)_.
    AppleTouchIcon(u32),
    /// Variant for generic entries.
    Icon(u32),
}

impl Favicon {
    fn get_html_helper(&self) -> io::Result<Vec<u8>> {
        let mut helper = Vec::with_capacity(self.keys.len() * 90);

        for entry in &self.keys {
            write!(
                helper,
                "<link rel=\"{0}\" type=\"image/png\" sizes=\"{1}x{1}\" href=\"{2}\">\n",
                entry.rel(), entry.as_ref(), entry.to_path_buff().display()
            )?;
        }

        Ok(helper)
    }
}

impl Icon for Favicon {
    type Key = FaviconKey;

    fn new() -> Self {
        Favicon {
            internal: PngSequence::new(),
            keys: Vec::with_capacity(STD_CAPACITY)
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        filter: F,
        source: &SourceImage,
        key: Self::Key,
    ) -> Result<(), Error<Self::Key>> {
        let path = key.to_path_buff();
        let png_entry = PathKey(*key.as_ref(), path);

        if let Err(err) = self.internal.add_entry(filter, source, png_entry) {
            Err(err.map(|_| key))
        } else {
            let _ = self.keys.push(key);
            self.keys.sort();
            Ok(())
        }
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let mut tar_builder = tar::Builder::new(w);
        self.internal.write_to_tar(&mut tar_builder)?;

        let helper = self.get_html_helper()?;
        write_data(&mut tar_builder, helper.as_ref(), "helper.html")
    }

    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<()> {
        if path.as_ref().is_file() {
            let mut file = File::create(path.as_ref())?;
            self.write(&mut file)
        } else {
            self.internal.save(path)?;

            let helper = self.get_html_helper()?;
            save_file(helper.as_ref(), path.as_ref(), "helper.html")
        }
    }
}

impl FaviconKey {
    #[inline]
    /// Returns the _rel_ 
    pub fn rel(&self) -> &str {
        match self {
            FaviconKey::AppleTouchIcon(_) => "apple-touch-icon-precomposed",
            FaviconKey::Icon(_) => "icon"
        }
    }

    #[inline]
    fn to_path_buff(self) -> PathBuf {
        match self {
            FaviconKey::AppleTouchIcon(size) => path!("icons/apple-touch-{}.png", size),
            FaviconKey::Icon(size) => path!("icons/favicon-{}.png", size),
        }
    }
}

impl AsRef<u32> for FaviconKey {
    fn as_ref(&self) -> &u32 {
        match self {
            FaviconKey::AppleTouchIcon(size) | FaviconKey::Icon(size) => size
        }
    }
}

impl PartialOrd for FaviconKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.as_ref().cmp(self.as_ref()) {
            Ordering::Equal => match (self, other) {
                (FaviconKey::AppleTouchIcon(_), FaviconKey::Icon(_)) => Some(Ordering::Greater),
                (FaviconKey::Icon(_), FaviconKey::AppleTouchIcon(_)) => Some(Ordering::Less),
                _ => Some(Ordering::Equal)
            },
            ord => Some(ord)
        }
    }
}

/// Helper function to append a buffer to a `.tar` file
fn write_data<W: Write>(builder: &mut tar::Builder<W>, data: &[u8], path: &str) -> io::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_cksum();

    builder.append_data::<&str, &[u8]>(&mut header, path, data)
}

/// Helper function to write a buffer to a location on disk.
fn save_file(data: &[u8], base_path: &Path, path: &str) -> io::Result<()> {
    let path = base_path.join(path);
    let mut file = File::create(path)?;

    file.write_all(data)
}
