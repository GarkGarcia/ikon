extern crate image;
extern crate tar;

use crate::{
    png_sequence::PngSequence, Error, Icon, PngEntry, SourceImage, STD_CAPACITY
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
pub struct FavIcon {
    internal: PngSequence,
    entries: Vec<FavIconKey>
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord)]
/// The _key type_ for `FavIcon`.
pub enum FavIconKey {
    /// Variant for 
    /// _[Apple touch icons](https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/ConfiguringWebApplications/ConfiguringWebApplications.html)_.
    AppleTouchIcon(u32),
    /// Variant for generic entries.
    Icon(u32),
}

impl FavIcon {
    fn get_html_helper(&self) -> io::Result<Vec<u8>> {
        let mut helper = Vec::with_capacity(self.entries.len() * 90);

        for entry in &self.entries {
            write!(
                helper,
                "<link rel=\"{0}\" type=\"image/png\" sizes=\"{1}x{1}\" href=\"{2}\">\n",
                entry.rel(), entry.as_ref(), entry.to_path_buff().display()
            )?;
        }

        Ok(helper)
    }
}

impl Icon<FavIconKey> for FavIcon {
    fn new() -> Self {
        FavIcon {
            internal: PngSequence::new(),
            entries: Vec::with_capacity(STD_CAPACITY)
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        filter: F,
        source: &SourceImage,
        entry: FavIconKey,
    ) -> Result<(), Error<FavIconKey>> {
        let path = entry.to_path_buff();
        let png_entry = PngEntry(*entry.as_ref(), path);

        if let Err(err) = self.internal.add_entry(filter, source, png_entry) {
            Err(err.map(|_| entry))
        } else {
            let _ = self.entries.push(entry);
            self.entries.sort();
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

impl FavIconKey {
    #[inline]
    fn rel(&self) -> &str {
        match self {
            FavIconKey::AppleTouchIcon(_) => "apple-touch-icon",
            FavIconKey::Icon(_) => "icon"
        }
    }

    #[inline]
    fn to_path_buff(self) -> PathBuf {
        match self {
            FavIconKey::AppleTouchIcon(size) => path!("icons/apple-touch-{0}x{0}-precomposed.png", size),
            FavIconKey::Icon(size) => path!("icons/favicon-{0}x{0}.png", size),
        }
    }
}

impl AsRef<u32> for FavIconKey {
    fn as_ref(&self) -> &u32 {
        match self {
            FavIconKey::AppleTouchIcon(size) | FavIconKey::Icon(size) => size
        }
    }
}

impl PartialOrd for FavIconKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.as_ref().cmp(self.as_ref()) {
            Ordering::Equal => match (self, other) {
                (FavIconKey::AppleTouchIcon(_), FavIconKey::Icon(_)) => Some(Ordering::Greater),
                (FavIconKey::Icon(_), FavIconKey::AppleTouchIcon(_)) => Some(Ordering::Less),
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
