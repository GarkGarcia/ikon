//! Structs for encoding _[favicons](https://en.wikipedia.org/wiki/Favicon)_.

extern crate image;
extern crate tar;

use crate::{resample, AsSize, Error, Icon, SourceImage, STD_CAPACITY};
use image::{png::PNGEncoder, ColorType, DynamicImage};
use resvg::usvg::{XmlIndent, XmlOptions};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Write},
    num::NonZeroU32,
    path::{Path, PathBuf},
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
    source_map: HashMap<ImageBuffer, Vec<u32>>,
    entries: Vec<u32>,
}

/// The _key type_ for `FavIcon`.
pub type FaviconKey = NonZeroU32;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ImageBuffer {
    Png(Vec<u8>),
    Svg(Vec<u8>),
}

impl Favicon {
    fn get_html_helper(&self) -> io::Result<Vec<u8>> {
        let mut helper = Vec::with_capacity(self.entries.len() * 180);
        let mut i = 0;

        for (buff, sizes) in self.entries() {
            write!(
                helper,
                "<link rel=\"icon\" type=\"{}\" sizes=\" ",
                buff.get_type()
            )?;

            for size in sizes {
                write!(&mut helper, "{0}x{0} ", size)?;
            }

            write!(
                helper,
                "\" href=\"icons/favicon@{}.{}\">\n",
                i,
                buff.get_extension()
            )?;

            i += 1;
        }

        Ok(helper)
    }

    /// Returns the content of `self.source_map` sorted by the minimum value
    /// of each value.
    fn entries(&self) -> Vec<(&ImageBuffer, &Vec<u32>)> {
        let len = self.entries.len();
        let mut output = Vec::with_capacity(len);

        for pair in &self.source_map {
            output.push(pair);
        }

        output.sort_by_key(|(_, sizes)| sizes[0]);
        output
    }
}

impl Icon for Favicon {
    type Key = FaviconKey;

    fn new() -> Self {
        Favicon {
            source_map: HashMap::with_capacity(STD_CAPACITY),
            entries: Vec::with_capacity(STD_CAPACITY),
        }
    }

    fn add_entry<F: FnMut(&SourceImage, u32) -> DynamicImage>(
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
        let entry = self.source_map.entry(buff).or_default();

        entry.push(size);
        entry.sort();
        self.entries.push(size);

        Ok(())
    }

    fn write<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let mut tar_builder = tar::Builder::new(w);
        let mut i = 0;

        for (buff, _) in self.entries() {
            let path = path!("icons/favicon@{}.{}", i, buff.get_extension());
            write_data(&mut tar_builder, buff.as_ref(), path)?;

            i += 1;
        }

        let helper = self.get_html_helper()?;
        write_data(&mut tar_builder, helper.as_ref(), path!("helper.html"))
    }

    fn save<P: AsRef<Path>>(&mut self, path: &P) -> io::Result<()> {
        if path.as_ref().is_file() {
            let mut file = File::create(path.as_ref())?;
            self.write(&mut file)
        } else {
            let mut i = 0;

            for (buff, _) in self.entries() {
                let path = path!("icons/favicon@{}.{}", i, buff.get_extension());
                save_file(buff.as_ref(), path.as_ref(), "helper.html")?;

                i += 1;
            }

            let helper = self.get_html_helper()?;
            save_file(helper.as_ref(), path.as_ref(), "helper.html")
        }
    }
}

impl AsSize for FaviconKey {
    fn as_size(&self) -> u32 {
        self.get()
    }
}

impl ImageBuffer {
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
}

impl AsRef<[u8]> for ImageBuffer {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Png(buff) | Self::Svg(buff) => buff.as_ref(),
        }
    }
}

fn get_image_buffer<F: FnMut(&SourceImage, u32) -> DynamicImage>(
    filter: F,
    source: &SourceImage,
    size: u32,
) -> Result<ImageBuffer, Error<FaviconKey>> {
    match source {
        SourceImage::Raster(_) => {
            get_png_buffer(resample::safe_filter(filter, source, size)?, size)
        }
        SourceImage::Svg(svg) => Ok(ImageBuffer::Svg(svg.to_string(XML_OPTS).into_bytes())),
    }
}

fn get_png_buffer(image: DynamicImage, size: u32) -> Result<ImageBuffer, Error<FaviconKey>> {
    let data = image.to_rgba().into_raw();
    // Encode the pixel data as PNG and store it in a Vec<u8>
    let mut output = Vec::with_capacity(data.len());
    let encoder = PNGEncoder::new(&mut output);
    encoder.encode(&data, size, size, ColorType::RGBA(8))?;

    Ok(ImageBuffer::Png(output))
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
fn save_file(data: &[u8], base_path: &Path, path: &str) -> io::Result<()> {
    let path = base_path.join(path);
    let mut file = File::create(path)?;

    file.write_all(data)
}
