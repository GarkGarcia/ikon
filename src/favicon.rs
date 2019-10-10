//! Structs for encoding _[favicons](https://en.wikipedia.org/wiki/Favicon)_.

extern crate image;
extern crate tar;

use crate::{resample, AsSize, Error, Icon, SourceImage};
use image::{png::PNGEncoder, ColorType, DynamicImage};
use resvg::usvg::{self, XmlIndent, XmlOptions};
use std::{
    convert::TryFrom,
    collections::hash_map::{HashMap, Entry},
    fs::{DirBuilder, File},
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

const XML_OPTS: XmlOptions = XmlOptions {
    use_single_quote: false,
    indent: XmlIndent::None,
    attributes_indent: XmlIndent::None,
};

const APPLE_TOUCH_SIZES: [u32;4] = [76, 120, 152, 180];

macro_rules! path {
    ($path: expr) => {
        PathBuf::from($path)
    };

    ($format: expr, $($arg: expr),*) => {
        PathBuf::from(format!($format, $($arg),*))
    };
}

#[derive(Clone)]
/// A comprehensive _favicon_ builder.
pub struct Favicon {
    pngs: HashMap<u32, Vec<u8>>,
    svgs: HashMap<Vec<u8>, Vec<u32>>,
    svg_entries: Vec<u32>,
    include_apple_touch_helper: bool,
    include_chrome_app_helper: bool
}

/// The _key type_ for `FavIcon`. Note that `Key(0)` represents
/// a _65536x65536_ entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key(pub u16);

#[derive(Clone, Debug, PartialEq, Eq)]
/// An iterator over the sizes of a `Favicon`
/// entry.
struct Sizes<'a> {
    sizes: Vec<&'a u32>,
    index: usize
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// An iterators over the entries of a `Favicon`,
/// sorted by size.
struct Entries<'a> {
    entries: Vec<(BufInfo<'a>, &'a Vec<u8>)>,
    index: usize
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Information about the file format and the
/// associated sizes of a file buffer.
enum BufInfo<'a> {
    Png(u32),
    Svg(&'a Vec<u32>),
}

impl Favicon {
    fn len(&self) -> usize {
        self.pngs.len() + self.svg_entries.len()
    }

    #[inline]
    pub fn apple_touch(&mut self, b: bool) -> &mut Self {
        self.include_apple_touch_helper = b;
        self
    }

    #[inline]
    pub fn chrome_app(&mut self, b: bool) -> &mut Self {
        self.include_chrome_app_helper = b;
        self
    }

    /// Returns a buffer containing _HTML_ link tags to assist on
    /// the creating of the icon.
    pub fn html_helper(&self) -> io::Result<Vec<u8>> {
        let mut helper = Vec::with_capacity(self.len() * 180);
        let mut i = 0;

        for (info, _) in self.entries() {
            let res_type = info.res_type();
            let extension = info.extension();
            let mut sizes = info.sizes();

            write_link_tag(&mut helper, "icon", res_type, i, extension, &mut sizes)?;

            if self.include_apple_touch_helper {
                let mut it = sizes
                    .filter(|size| APPLE_TOUCH_SIZES.contains(&size));

                write_link_tag(
                    &mut helper,
                    "apple-touch-icon-precomposed",
                    res_type,
                    i,
                    extension,
                    &mut it
                )?;
            }

            i += 1;
        }

        if self.include_chrome_app_helper {
            write!(helper, "<link rel=\"manifest\" href=\"manifest.json\">\n")?;
        }

        Ok(helper)
    }

    /// Returns the `Favicon`'s entries sorted by size.
    fn entries(&self) -> Entries<'_> {
        let mut entries = Vec::with_capacity(self.len());

        for (&size, buf) in &self.pngs {
            entries.push((BufInfo::Png(size), buf));
        }

        for (buf, sizes) in &self.svgs {
            entries.push((BufInfo::Svg(sizes), buf));
        }

        // Sort by the minimun size associated with the file
        entries.sort_by_key(|(info, _)| info.min_size());
        Entries { index: 0, entries }
    }

    #[inline]
    fn add_raster<F: FnMut(&SourceImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        key: Key
    ) -> Result<(), Error<Key>> {
        let size = key.as_size();
        let icon = resample::apply(filter, source, size)?;
        let buf = get_png_buffer(&icon, size)?;

        match self.pngs.entry(size) {
            Entry::Occupied(_) => Err(Error::AlreadyIncluded(key)),
            Entry::Vacant(entry) => {
                let _ = entry.insert(buf);
                Ok(())
            }
        }
    }

    #[inline]
    fn add_svg(&mut self, svg: &usvg::Tree, key: Key) -> Result<(), Error<Key>> {
        let size = key.as_size();

        if self.svg_entries.contains(&size) {
            Err(Error::AlreadyIncluded(key))
        } else {
            let buf = svg.to_string(XML_OPTS).into_bytes();
            let entry = self.svgs.entry(buf).or_default();

            entry.push(size);
            self.svg_entries.push(size);

            Ok(())
        }
    }

    fn save_to_dir<P: AsRef<Path>>(&self, base_path: &P) -> io::Result<()> {
        let container = base_path.as_ref().join("icons/");

        if !container.exists() {
            let mut builder = DirBuilder::new();
            builder.recursive(true).create(container)?;
        }

        let mut i = 0;

        for (info, buf) in self.entries() {
            let path = path!("icons/favicon-{}.{}", i, info.extension());
            save_file(buf.as_ref(), base_path, &path)?;

            i += 1;
        }

        let helper = self.html_helper()?;
        save_file(helper.as_ref(), base_path, &"helper.html")
    }
}

impl Icon for Favicon {
    type Key = Key;

    fn with_capacity(capacity: usize) -> Self {
        Favicon {
            pngs: HashMap::with_capacity(capacity),
            svgs: HashMap::with_capacity(capacity),
            svg_entries: Vec::with_capacity(capacity),
            include_apple_touch_helper: false,
            include_chrome_app_helper: false
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        key: Self::Key,
    ) -> Result<(), Error<Self::Key>> {
        match source {
            SourceImage::Raster(_) => self.add_raster(filter, source, key),
            SourceImage::Svg(svg) => self.add_svg(svg, key)
        }
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let mut tar_builder = tar::Builder::new(w);
        let mut i = 0;

        for (info, buf) in self.entries() {
            let path = path!("icons/favicon-{}.{}", i, info.extension());
            write_data(&mut tar_builder, buf.as_ref(), path)?;

            i += 1;
        }

        let helper = self.html_helper()?;
        write_data(&mut tar_builder, helper.as_ref(), path!("helper.html"))
    }

    fn save<P: AsRef<Path>>(&mut self, base_path: &P) -> io::Result<()> {
        if base_path.as_ref().is_file() {
            let mut file = File::create(base_path.as_ref())?;
            self.write(&mut file)
        } else {
            self.save_to_dir(base_path)
        }
    }
}

impl AsSize for Key {
    fn as_size(&self) -> u32 {
        if self.0 == 0 {
            65536
        } else {
            self.0 as u32
        }
    }
}

impl TryFrom<u32> for Key {
    type Error = io::Error;

    fn try_from(val: u32) -> io::Result<Self> {
        match val {
            65536 => Ok(Key(0)),
            0 => Err(io::Error::from(io::ErrorKind::InvalidInput)),
            n if n < 65536 => Ok(Key(n as u16)),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput))
        }
    }
}

impl FromStr for Key {
    type Err = io::Error;

    fn from_str(s: &str) -> io::Result<Self> {
        match s {
            "65536" => Ok(Key(0)),
            "0" => Err(io::Error::from(io::ErrorKind::InvalidInput)),
            _ => s
                .parse::<u16>()
                .map(Key)
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }
}

impl<'a> From<Vec<&'a u32>> for Sizes<'a> {
    fn from(sizes: Vec<&'a u32>) -> Self {
        Self { index: 0, sizes }
    }
}

impl<'a> Iterator for Sizes<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.sizes.len() {
            let output = Some(*self.sizes[self.index]);
            self.index += 1;

            output
        } else {
            None
        }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = (BufInfo<'a>, &'a Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entries.len() {
            let output = Some(self.entries[self.index]);
            self.index += 1;

            output
        } else {
            None
        }
    }
}

impl<'a> BufInfo<'a> {
    #[inline]
    fn res_type(&self) -> &str {
        match self {
            Self::Png(_) => "image/png",
            Self::Svg(_) => "image/svg+xml",
        }
    }

    #[inline]
    fn extension(&self) -> &str {
        match self {
            Self::Png(_) => "png",
            Self::Svg(_) => "svg",
        }
    }

    #[inline]
    /// Returns the smallest size associated with this entry.
    fn min_size(&self) -> u32 {
        match self {
            Self::Png(size) => *size,
            Self::Svg(sizes) => sizes[0],
        }
    }

    fn sizes(&self) -> Sizes<'_> {
        match self {
            BufInfo::Png(size) => Sizes::from(vec![size]),
            BufInfo::Svg(sizes) => {
                let mut output = Vec::with_capacity(sizes.len());

                for size in *sizes {
                    output.push(size);
                }

                Sizes::from(output)
            }
        }
    }
}

fn write_link_tag<W: Write, I: Iterator<Item = u32>>(
    w: &mut W,
    rel: &str,
    res_type: &str,
    index: usize,
    extension: &str,
    it: &mut I
) -> io::Result<()> {
    if let Some(size) = it.next() {
        write!(
            w,
            "<link rel=\"{0}\" type=\"{1}\" sizes=\"{2}x{2}",
            rel, res_type, size
        )?;

        while let Some(size) = it.next() {
            write!(w, " {0}x{0}", size)?;
        }

        write!(
            w,
            "\" href=\"icons/favicon-{}.{}\">\n",
            index,
            extension
        )?;
    }

    Ok(())
}

fn get_png_buffer(image: &DynamicImage, size: u32) -> Result<Vec<u8>, Error<Key>> {
    let data = image.to_rgba().into_raw();
    // Encode the pixel data as PNG and store it in a Vec<u8>
    let mut output = Vec::with_capacity(data.len());
    let encoder = PNGEncoder::new(&mut output);
    encoder.encode(&data, size, size, ColorType::RGBA(8))?;

    Ok(output)
}

/// Helper function to append a buffer to a `.tar` file
fn write_data<W: Write>(
    builder: &mut tar::Builder<W>,
    data: &[u8],
    path: PathBuf,
) -> io::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_cksum();

    builder.append_data::<PathBuf, &[u8]>(&mut header, path, data)
}

/// Helper function to write a buffer to a location on disk.
fn save_file<P1: AsRef<Path>, P2: AsRef<Path>>(
    data: &[u8],
    base_path: &P1,
    path: &P2,
) -> io::Result<()> {
    let path = base_path.as_ref().join(path);
    let mut file = File::create(path)?;

    file.write_all(data)
}
