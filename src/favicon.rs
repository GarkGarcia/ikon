extern crate image;
extern crate tar;

use crate::{
    ico::Ico, png_sequence::PngSequence, Error, Icon, PngEntry, Size, SourceImage, STD_CAPACITY,
};
use image::DynamicImage;
use resvg::usvg::{Tree, XmlOptions};
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

const SHORTCUT_SIZES: [Size; 3] = [Size(16), Size(32), Size(48)];

#[derive(Clone, Debug)]
/// A comprehencive _favicon_ builder.
pub struct FavIcon {
    raw_sequence: PngSequence,
    html_helper: Vec<u8>,
    ms_tile_color: Option<Color>,
    shortcut_icon: Option<Vec<u8>>,
    safari_pinned_tab_icon: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// The _entry type_ for `FavIcon`.
pub enum FavIconEntry {
    /// Variant for 
    /// _[Safari web-app icons](https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/ConfiguringWebApplications/ConfiguringWebApplications.html)_.
    AppleTouchIcon,
    /// Variant for generic entries.
    Icon(u32),
    /// Variant for configuring
    /// _[IE app icons](https://docs.microsoft.com/en-us/previous-versions/windows/internet-explorer/ie-developer/platform-apis/dn320426(v=vs.85))_.
    MsApplicationIcon(Color),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// A simple representation of an RGB color.
pub struct Color(u8, u8, u8);

impl FavIcon {
    /// Adds an entry for a 
    /// _[shortcut icon](https://developer.mozilla.org/en-US/docs/Learn/HTML/Introduction_to_HTML/The_head_metadata_in_HTML#Adding_custom_icons_to_your_site)_
    /// , creating a `.ico` file containing a _16x16_ entry, a _32x32_ entry and
    /// a _48x48_ entry.
    /// 
    /// # Return Value
    /// 
    /// * Returns `Err(Error::Io(io::Error::from(io::ErrorKind::AlreadyExists)))`
    ///   if the icon already contains a _shortcut icon_.
    /// * Returns `Err(_)` if the construction of the `.ico` file fails or if the
    ///   icon's html helper can't be updated.
    /// * Returns `Ok(())` otherwise.
    pub fn add_shortcut_icon<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        filter: F,
        source: &SourceImage,
    ) -> Result<(), Error<FavIconEntry>> {
        if let Some(_) = self.shortcut_icon {
            return Err(Error::Io(io::Error::from(io::ErrorKind::AlreadyExists)));
        }

        let mut ico = Ico::new();

        if let Err(err) = ico.add_entries(filter, source, SHORTCUT_SIZES.to_vec()) {
            match err {
                Error::AlreadyIncluded(_) => panic!("This shouldn't happen."),
                _ => return Err(err.map(|_| unreachable!())),
            }
        }

        let mut shortcut = Vec::with_capacity(15_000);
        ico.write(&mut shortcut)?;

        self.shortcut_icon = Some(shortcut);
        write!(
            self.html_helper,
            "\n<link rel=\"shortcut icon\" type=\"image/x-icon\" href=\"icons/favicon.ico\"/>"
        )
        .map_err(|err| Error::Io(err))
    }

    /// Add an entry for a 
    /// _[Safari pinned tab icon](https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/pinnedTabs/pinnedTabs.html)_.
    /// 
    /// # Return Value
    /// 
    /// * Returns `Err(Error::Io(io::Error::from(io::ErrorKind::AlreadyExists)))`
    ///   if the icon already contains a _Safari pinned tab icon_.
    /// * Returns `Err(_)` if the icon's html helper can't be updated.
    /// * Returns `Ok(())` otherwise.
    pub fn add_safari_pinned_tab_icon(
        &mut self,
        source: &Tree,
        color: &Color,
    ) -> Result<(), Error<FavIconEntry>> {
        if let Some(_) = self.safari_pinned_tab_icon {
            return Err(Error::Io(io::Error::from(io::ErrorKind::AlreadyExists)));
        }

        self.safari_pinned_tab_icon = Some(source.to_string(XmlOptions::default()));
        write!(
            self.html_helper,
            "\n<link rel=\"mask-icon\" href=\"icons/safari-pinned-tab.svg\" color=\"{}\"/>",
            color.to_hex()
        )
        .map_err(|err| Error::Io(err))
    }

    #[inline]
    fn append_helper(&mut self, entry: &FavIconEntry) -> io::Result<()> {
        match entry {
            FavIconEntry::AppleTouchIcon => self.append_apple_helper(),
            FavIconEntry::Icon(size) => self.append_icon_helper(*size),
            FavIconEntry::MsApplicationIcon(color) => self.append_ms_app_helper(*color),
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
    fn append_ms_app_helper(&mut self, color: Color) -> io::Result<()> {
        self.ms_tile_color = Some(color);

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
            shortcut_icon: None,
            safari_pinned_tab_icon: None,
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
        &mut self,
        filter: F,
        source: &SourceImage,
        entry: FavIconEntry,
    ) -> Result<(), Error<FavIconEntry>> {
        let label = PngEntry(*entry.as_ref(), entry.to_path_buff());

        if let Err(err) = self.raw_sequence.add_entry(filter, source, label) {
            return Err(err.map(|_| entry));
        }

        self.append_helper(&entry).map_err(|err| Error::Io(err))
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let mut tar_builder = tar::Builder::new(w);

        write_data(&mut tar_builder, self.html_helper.as_ref(), "helper.html")?;

        if let Some(color) = self.ms_tile_color {
            let browserconfig = get_ms_browserconfig(color);
            write_data(
                &mut tar_builder,
                browserconfig.as_ref(),
                "icons/browserconfig.xml",
            )?;
        }

        if let Some(buff) = &self.shortcut_icon {
            write_data(&mut tar_builder, buff.as_ref(), "icons/favicon.ico")?;
        }

        if let Some(svg) = &self.safari_pinned_tab_icon {
            write_data(
                &mut tar_builder,
                svg.clone().into_bytes().as_ref(),
                "icons/safari-pinned-tab.svg"
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

            if let Some(color) = self.ms_tile_color {
                let browserconfig = get_ms_browserconfig(color);

                save_file(
                    browserconfig.as_ref(),
                    path.as_ref(),
                    "icons/browserconfig.xml",
                )?;
            }

            if let Some(buff) = &self.shortcut_icon {
                save_file(buff.as_ref(), path.as_ref(), "icons/favicon.ico")?;
            }

            if let Some(svg) = &self.safari_pinned_tab_icon {
                save_file(
                    svg.clone().into_bytes().as_ref(),
                    path.as_ref(),
                    "icons/safari-pinned-tab.svg"
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
            FavIconEntry::MsApplicationIcon(_) => PathBuf::from("icons/mstile-150x150.png"),
        }
    }
}

impl AsRef<u32> for FavIconEntry {
    fn as_ref(&self) -> &u32 {
        match self {
            FavIconEntry::AppleTouchIcon => &180,
            FavIconEntry::Icon(size) => size,
            FavIconEntry::MsApplicationIcon(_) => &150,
        }
    }
}

impl Color {
    /// Display the color as _css-styled_ hex `String`.
    fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
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

#[inline]
fn get_ms_browserconfig(color: Color) -> Vec<u8> {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>
<browserconfig>
    <msapplication>
        <tile>
            <square150x150logo src=\"mstile-150x150.png\"/>
            <TileColor>#{}</TileColor>
        </tile>
    </msapplication>
</browserconfig>",
        color.to_hex()
    )
    .into_bytes()
}
