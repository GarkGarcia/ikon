extern crate image;
extern crate tar;

use crate::{png_sequence::PngSequence, Error, FileLabel, Icon, SourceImage, STD_CAPACITY};
use image::DynamicImage;
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

/// A collection of entries stored in a single `.tar` file.
#[derive(Clone, Debug)]
pub struct FavIcon {
    raw_sequence: PngSequence,
    html_helper: Vec<u8>,
    ms_tile_color: Option<(u8, u8, u8)>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FavIconEntry {
    AppleTouchIcon,
    Icon(u32),
    MsApplicationIcon(u8, u8, u8),
}

impl FavIcon {
    #[inline]
    fn append_helper(&mut self, entry: &FavIconEntry) -> io::Result<()> {
        match entry {
            FavIconEntry::AppleTouchIcon => self.append_apple_helper(),
            FavIconEntry::Icon(size) => self.append_icon_helper(*size),
            FavIconEntry::MsApplicationIcon(r, g, b) => self.append_ms_app_helper(*r, *g, *b),
        }
    }

    #[inline]
    fn append_apple_helper(&mut self) -> io::Result<()> {
        write!(
            self.html_helper,
            "\n<link rel=\"apple-touch-icon\" sizes=\"180x180\" href=\"icons/apple-touch-icon.png\"/>"
        )
    }
    #[inline]
    fn append_icon_helper(&mut self, size: u32) -> io::Result<()> {
        write!(
            self.html_helper,
            "\n<link rel=\"icon\" sizes=\"{0}x{0}\" type=\"image/png\" href=\"icons/favicon-{0}x{0}.png\"/>",
            size
        )
    }
    #[inline]
    fn append_ms_app_helper(&mut self, r: u8, g: u8, b: u8) -> io::Result<()> {
        self.ms_tile_color = Some((r, g, b));

        write!(
            self.html_helper,
            "\n<meta name=\"msapplication-config\" href=\"icons/browserconfig.xml\">"
        )
    }
}

impl Icon<FavIconEntry> for FavIcon {
    fn new() -> Self {
        FavIcon {
            raw_sequence: PngSequence::new(),
            html_helper: Vec::with_capacity(STD_CAPACITY * 90),
            ms_tile_color: None,
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        filter: F,
        source: &SourceImage,
        entry: FavIconEntry,
    ) -> Result<(), Error<FavIconEntry>> {
        let label = FileLabel(*entry.as_ref(), entry.to_path_buff());

        if let Err(err) = self.raw_sequence.add_entry(filter, source, label) {
            return Err(file_label_to_favicon_entry_err(err, entry));
        }

        self.append_helper(&entry).map_err(|err| Error::Io(err))
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let mut tar_builder = tar::Builder::new(w);

        write_data(&mut tar_builder, self.html_helper.as_ref(), "helper.html")?;

        if let Some((r, g, b)) = self.ms_tile_color {
            let browserconfig = get_ms_browserconfig(r, g, b);
            write_data(
                &mut tar_builder,
                browserconfig.as_ref(),
                "icons/browserconfig.xml"
            )?;
        }

        self.raw_sequence.write_to_tar(&mut tar_builder)
    }

    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<()> {
        if path.as_ref().is_file() {
            let mut file = File::create(path.as_ref())?;
            self.write(&mut file)
        } else {
            save_file(self.html_helper.as_ref(), path.as_ref(), "helper.html")?;

            if let Some((r, g, b)) = self.ms_tile_color {
                let browserconfig = get_ms_browserconfig(r, g, b);

                save_file(
                    browserconfig.as_ref(),
                    path.as_ref(),
                    "icons/browserconfig.xml"
                )?;
            }

            self.raw_sequence.save(path)
        }
    }
}

impl FavIconEntry {
    #[inline]
    fn to_path_buff(self) -> PathBuf {
        match self {
            FavIconEntry::AppleTouchIcon => PathBuf::from("icons/apple-touch-icon.png"),
            FavIconEntry::Icon(size) => PathBuf::from(format!("icons/favicon-{0}x{0}.png", size)),
            FavIconEntry::MsApplicationIcon(_, _, _) => PathBuf::from("icons/mstile-150x150.png"),
        }
    }
}

impl AsRef<u32> for FavIconEntry {
    fn as_ref(&self) -> &u32 {
        match self {
            FavIconEntry::AppleTouchIcon => &180,
            FavIconEntry::Icon(size) => size,
            FavIconEntry::MsApplicationIcon(_, _, _) => &150,
        }
    }
}

/// Helper function to append a buffer to a `.tar` file
fn write_data<W: Write>(
    builder: &mut tar::Builder<W>,
    data: &[u8],
    path: &str
) -> io::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_cksum();

    builder.append_data::<&str, &[u8]>(
        &mut header,
        path,
        data,
    )
}

/// Helper function to write a buffer to a location on disk.
fn save_file(data: &[u8], base_path: &Path, path: &str) -> io::Result<()> {
    let path = base_path.join(path);
    let mut file = File::create(path)?;

    file.write_all(data)
}

#[inline]
// Converts a `Error<FileLabel>` to a `Error<FavIconEntry>`
fn file_label_to_favicon_entry_err(
    err: Error<FileLabel>,
    entry: FavIconEntry,
) -> Error<FavIconEntry> {
    match err {
        Error::AlreadyIncluded(_) => Error::AlreadyIncluded(entry),
        Error::InvalidDimensions(size) => Error::InvalidDimensions(size),
        Error::Io(err) => Error::Io(err),
        Error::MismatchedDimensions(e, g) => Error::MismatchedDimensions(e, g),
    }
}

#[inline]
fn get_ms_browserconfig(r: u8, g: u8, b: u8) -> Vec<u8> {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>
<browserconfig>
    <msapplication>
        <tile>
            <square150x150logo src=\"mstile-150x150.png\"/>
            <TileColor>#{:02x}{:02x}{:02x}</TileColor>
        </tile>
    </msapplication>
</browserconfig>",
        r, g, b
    )
    .into_bytes()
}
