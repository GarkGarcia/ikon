//! Structs for encoding _[favicons](https://en.wikipedia.org/wiki/Favicon)_.

extern crate image;
extern crate tar;

use crate::{resample, AsSize, Error, Icon, SourceImage};
use image::{png::PNGEncoder, ColorType, DynamicImage};
use resvg::usvg::{XmlIndent, XmlOptions};
use std::{
    convert::TryFrom,
    collections::hash_map::{Entry, HashMap, OccupiedEntry, VacantEntry},
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
    source_map: HashMap<Vec<u8>, BuffInfo>,
    entries: Vec<u32>,
}

/// The _key type_ for `FavIcon`. Note that `Key(0)` represents
/// a _65536x65536_ entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key(pub u16);

#[derive(Clone, Debug, PartialEq, Eq)]
/// Information about the file format and the
/// associated sizes of a file buffer.
enum BuffInfo {
    Png(u32),
    Svg(Vec<u32>),
}

impl Favicon {
    /// Returns a buffer containing _HTML_ link tags to assist on
    /// the creating of the icon.
    pub fn html_helper(&self) -> io::Result<Vec<u8>> {
        let mut helper = Vec::with_capacity(self.entries.len() * 180);
        let mut i = 0;

        for (_, info) in self.entries() {
            write!(
                helper,
                "<link rel=\"icon\" type=\"{}\" sizes=\" ",
                info.get_type()
            )?;

            info.write_sizes(&mut helper)?;

            write!(
                helper,
                "\" href=\"icons/favicon-{}.{}\">\n",
                i,
                info.get_extension()
            )?;

            i += 1;
        }

        Ok(helper)
    }

    /// Returns the content of `self.source_map` sorted by the minimum value
    /// of each value.
    fn entries(&self) -> Vec<(&Vec<u8>, &BuffInfo)> {
        let len = self.entries.len();
        let mut output = Vec::with_capacity(len);

        for pair in &self.source_map {
            output.push(pair);
        }

        // Sort by the minimun size associated with the file
        output.sort_by_key(|(_, entry)| entry.get_min_size());
        output
    }
}

impl Icon for Favicon {
    type Key = Key;

    fn with_capacity(capacity: usize) -> Self {
        Favicon {
            source_map: HashMap::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &SourceImage,
        key: Self::Key,
    ) -> Result<(), Error<Self::Key>> {
        let size = key.as_size();

        if self.entries.contains(&size) {
            return Err(Error::AlreadyIncluded(key));
        }

        let buff = get_image_buffer(filter, source, size)?;

        match self.source_map.entry(buff) {
            Entry::Occupied(entry) => insert_occupied(entry, size),
            Entry::Vacant(entry) => insert_vacant(entry, source, size),
        }

        self.entries.push(size);
        Ok(())
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let mut tar_builder = tar::Builder::new(w);
        let mut i = 0;

        for (buff, info) in self.entries() {
            let path = path!("icons/favicon-{}.{}", i, info.get_extension());
            write_data(&mut tar_builder, buff.as_ref(), path)?;

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
            let container = base_path.as_ref().join("icons/");

            if !container.exists() {
                let mut builder = DirBuilder::new();
                builder.recursive(true).create(container)?;
            }

            let mut i = 0;

            for (buff, info) in self.entries() {
                let path = path!("icons/favicon-{}.{}", i, info.get_extension());
                save_file(buff.as_ref(), base_path, &path)?;

                i += 1;
            }

            let helper = self.html_helper()?;
            save_file(helper.as_ref(), base_path, &"helper.html")
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

impl BuffInfo {
    #[inline]
    fn get_type(&self) -> &str {
        match self {
            Self::Png(_) => "image/png",
            Self::Svg(_) => "image/svg+xml",
        }
    }

    #[inline]
    fn get_extension(&self) -> &str {
        match self {
            Self::Png(_) => "png",
            Self::Svg(_) => "svg",
        }
    }

    #[inline]
    /// Returns the smallest size associated with this entry.
    fn get_min_size(&self) -> u32 {
        match self {
            Self::Png(size) => *size,
            Self::Svg(sizes) => sizes[0],
        }
    }

    #[inline]
    fn write_sizes(&self, w: &mut Vec<u8>) -> io::Result<()> {
        match self {
            BuffInfo::Png(size) => write!(w, "{0}x{0} ", size)?,
            BuffInfo::Svg(sizes) => {
                for size in sizes {
                    write!(w, "{0}x{0} ", size)?;
                }
            }
        }

        Ok(())
    }
}

#[inline]
fn insert_vacant<'a>(entry: VacantEntry<'a, Vec<u8>, BuffInfo>, source: &SourceImage, size: u32) {
    let _ = match source {
        SourceImage::Raster(_) => entry.insert(BuffInfo::Png(size)),
        SourceImage::Svg(_) => entry.insert(BuffInfo::Svg(vec![size])),
    };
}

#[inline]
fn insert_occupied<'a>(entry: OccupiedEntry<'a, Vec<u8>, BuffInfo>, size: u32) {
    if let BuffInfo::Svg(ref mut vec) = entry.into_mut() {
        vec.push(size);
        vec.sort();
    }
}

fn get_image_buffer<F: FnMut(&SourceImage, u32) -> io::Result<DynamicImage>>(
    filter: F,
    source: &SourceImage,
    size: u32,
) -> Result<Vec<u8>, Error<Key>> {
    match source {
        SourceImage::Raster(_) => {
            get_png_buffer(resample::safe_filter(filter, source, size)?, size)
        }
        SourceImage::Svg(svg) => Ok(svg.to_string(XML_OPTS).into_bytes()),
    }
}

fn get_png_buffer(image: DynamicImage, size: u32) -> Result<Vec<u8>, Error<Key>> {
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
