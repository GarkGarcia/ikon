//! Structs for encoding _[favicons](https://en.wikipedia.org/wiki/Favicon)_.

extern crate image;
extern crate tar;

use crate::{encode, resample, AsSize, IconError, Icon, Image};
use image::DynamicImage;
use resvg::usvg;
use std::{
    convert::TryFrom,
    collections::{hash_map::{HashMap, Entry}, btree_set::BTreeSet},
    fs::{DirBuilder, File},
    io::{self, Write},
    path::{Path, PathBuf},
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
    svg_entries: BTreeSet<u32>,
    include_apple_touch_helper: bool,
    include_pwa_helper: bool
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
    #[inline]
    /// Indicates that the outputted _html-helper_ should contain link
    /// tags for _[apple-touch icons](https://mathiasbynens.be/notes/touch-icons)_.
    /// 
    /// This option defaults to `false`.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let fav = Favicon::new().apple_touch(true);
    /// ```
    /// 
    /// ```xml
    /// <link rel="apple-touch-icon-precomposed" sizes="196x196" href="icons/favicon-0.png">
    /// <link rel="icon" sizes="196x196" href="icons/favicon-0.png">
    /// ```
    pub fn apple_touch(&mut self, b: bool) -> &mut Self {
        self.include_apple_touch_helper = b;
        self
    }

    #[inline]
    /// Indicates that the output of `self.write` or
    /// `self.save` should contain a `manifest.webmanifest`
    /// file to assist on creating
    /// _[PWA icons](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Installable_PWAs)_.
    /// 
    /// This option defaults to `false`.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let fav = Favicon::new().web_app(true);
    /// ```
    /// 
    /// ```xml
    /// ...
    /// <link rel="manifest" href="manifest.webmanifest">
    /// ```
    /// 
    /// ```json
    /// {
    ///     icons: [
    ///         {
    ///             src: "icons/favicon-0.png",
    ///             sizes: "196x196",
    ///             type: "image/png"
    ///         }
    ///     ],
    /// }
    /// ```
    pub fn web_app(&mut self, b: bool) -> &mut Self {
        self.include_pwa_helper = b;
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

            write!(helper, "<link rel=\"icon\" type=\"{}\" sizes=\"", res_type)?;
            info.write_sizes(&mut helper, |_| true)?;
            write!(helper, "\" href=\"icons/favicon-{}.{}\">\n", i, extension)?;

            if self.include_apple_touch_helper {
                write!(helper, "<link rel=\"apple-touch-icon-precomposed\" type=\"{}\" sizes=\"", res_type)?;
                info.write_sizes(&mut helper, |size| APPLE_TOUCH_SIZES.contains(&size))?;
                write!(helper, "\" href=\"icons/favicon-{}.{}\">\n", i, extension)?;
            }

            i += 1;
        }

        Ok(helper)
    }

    /// Returns a buffer containing a _JSON_ helper for web manisfests.
    pub fn manifest(&self) -> io::Result<Vec<u8>> {
        // TODO Preallocate this
        let mut manifest = Vec::new();
        let mut i = 0;

        write!(manifest, "{{\n    \"icons\": [\n")?;

        for (info, _) in self.entries() {
            write!(
                manifest,
                "        {{\n            \"src\": \"icons/favicon-{}.{}\",\n            \"sizes\": \"",
                i,
                info.extension()
            )?;

            info.write_sizes(&mut manifest, |_| true)?;

            write!(
                manifest,
                "\",\n            \"type\": \"{}\"\n        }},\n",
                info.res_type()
            )?;

            i += 1;
        }

        // Remove the last comma
        manifest.pop();
        manifest.pop();

        write!(manifest, "\n    ]\n}}")?;

        Ok(manifest)
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
    /// Adds a raster entry.
    fn add_raster(
        &mut self,
        source: &DynamicImage,
        key: Key
    ) -> Result<(), IconError<Key>> {
        let size = key.as_size();

        match self.pngs.entry(size) {
            Entry::Occupied(_) => Err(IconError::AlreadyIncluded(key)),
            Entry::Vacant(entry) => {
                // TODO Size this buffer
                let buf = encode::png(source)?;
                entry.insert(buf);

                Ok(())
            }
        }
    }

    #[inline]
    /// Adds an SVG entry.
    fn add_svg(&mut self, svg: &usvg::Tree, key: Key) -> Result<(), IconError<Key>> {
        let size = key.as_size();

        if !self.svg_entries.insert(size) {
            Err(IconError::AlreadyIncluded(key))
        } else {
            let buf = encode::svg(svg);
            let entry = self.svgs.entry(buf).or_default();

            entry.push(size);
            Ok(())
        }
    }

    /// Saves the _favicon_ to a directory.
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

        let mut helper = self.html_helper()?;

        if self.include_pwa_helper {
            write!(helper, "<link rel=\"manifest\" href=\"app.webmanifest\">\n")?;

            let manifest = self.manifest()?;
            save_file(manifest.as_ref(), base_path, &"app.webmanifest")?;
        }

        save_file(helper.as_ref(), base_path, &"helper.html")
    }
}

impl Icon for Favicon {
    type Key = Key;

    fn with_capacity(capacity: usize) -> Self {
        Favicon {
            pngs: HashMap::with_capacity(capacity),
            svgs: HashMap::new(),
            svg_entries: BTreeSet::new(),
            include_apple_touch_helper: false,
            include_pwa_helper: false
        }
    }

    fn len(&self) -> usize {
        self.pngs.len() + self.svg_entries.len()
    }

    fn add_entry<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &Image,
        key: Self::Key,
    ) -> Result<(), IconError<Self::Key>> {
        match source {
            Image::Raster(ras) => self.add_raster(&resample::apply(filter, ras, key.as_size())?, key),
            Image::Svg(svg) => self.add_svg(&svg, key)
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

        let mut helper = self.html_helper()?;

        if self.include_pwa_helper {
            write!(helper, "<link rel=\"manifest\" href=\"app.webmanifest\">\n")?;

            let manifest = self.manifest()?;
            write_data(&mut tar_builder, manifest.as_ref(), path!("app.webmanifest"))?;
        }

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

    fn write_sizes<W: Write, P: FnMut(&u32) -> bool>(
        &self,
        w: &mut W,
        pred: P
    ) -> io::Result<()> {
        let mut it = self.sizes().filter(pred);

        if let Some(size) = it.next() {
            write!(w, "{0}x{0}", size)?;
    
            while let Some(size) = it.next() {
                write!(w, " {0}x{0}", size)?;
            }
        } else {
            panic!("`self.sizes()` should not be empty");
        }
    
        Ok(())
    }
}

#[inline]
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

#[inline]
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
