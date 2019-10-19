# IconBaker

[![Crate](https://img.shields.io/crates/v/icon_baker.svg)](https://crates.io/crates/icon_baker)
[![API](https://docs.rs/icon_baker/badge.svg)](https://docs.rs/icon_baker)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.37+-lightgray.svg)
![Downloads](https://img.shields.io/crates/d/icon_baker)
[![License](https://img.shields.io/github/license/GarkGarcia/icon_baker)](https://github.com/GarkGarcia/icon_baker/blob/master/LICENSE)

A simple solution for encoding common icon file-formats, such as `.ico`, `.icns` and _favicon_. 

This crate is mostly a wrapper for other libraries, unifying existing APIs into a single, cohesive
interface. It serves as **[IconPie's](https://github.com/GarkGarcia/icon-pie)** internal library.

# Overview

An _icon_ consists of a map between _keys_ and _images_. An _entry_ is a _key-value_ pair contained
in an _icon_.

**IconBaker** simply automates the process of re-scaling _images_, creating _entries_ and combining
them into an _icon_.

## Keys

Each _icon_ format is associated with a particular _key type_, which determines how
_entries_ are labeled. Each _key_ can only be associated with a single _image_.

For example, _icon_ formats that only differentiate _entries_ by the dimensions of their associated
_images_ are labeled by _positive integers_, such as the `.ico` and `.icns` file-formats.

On the other hand, _icon_ formats that distinguish their _entries_ by 
_[path](https://en.wikipedia.org/wiki/Path_%28computing%29)_, such as _png sequeces_ and
_[FreeDesktop icon themes](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html)_
, are labeled by _path_.

Note that, since the dimensions
of the _images_ contained in an _entry_ are dictated by their associated _entries_, every _key_
must be convertible to a _positive integers_. Therefore, all _key types_ are required to implement
`AsRef<u32>`.

## Resampling

Pictures are scaled using resampling filters, which are represented by _functions that take a source_ 
_image and a size and return a re-scaled image_.

This allows the users of this crate to provide their custom resampling filters. Common resampling 
filters are provided in the 
[`resample`](https://docs.rs/icon_baker/2.2.0/icon_baker/resample/index.html) module.

# Examples

## General Usage

The `Icon::add_entry` can be used to automatically resample
_source images_ and converts them to _entries_ in an icon.

```rust
use icon_baker::{ico::{Ico, Key}, SourceImage, Icon, Error};
  
fn example() -> Result<(), Error> {
    let icon = Ico::new();
    let src = SourceImage::open("image.svg")?;

    icon.add_entry(resample::linear, &img, Key(32))
}
```

## Writing to Disk

Implementors of the `Icon` trait can be written to any object
that implements `io::Write` with the `Icon::write` method.

```rust
use icon_baker::favicon::Favicon;
use std::{io, fs::File};
 
fn example() -> io::Result<()> {
    let icon = Favicon::new();

    // Process the icon ...

    let file = File::create("out.icns")?;
    icon.write(file)
}
```

Alternatively, icons can be directly written to a file on
disk with `Icon::save` method.

```rust
use icon_baker::favicon::Favicon;
use std::{io, fs::File};
 
fn example() -> io::Result<()> {
    let icon = Favicon::new();

    /* Process the icon */

    icon.save("./output/")
}
```

# Support

## Icon Formats

These are the output formats **IconBaker** natively supports. Be aware that custom output types can 
be created using the [`Icon`](https://docs.rs/icon_baker/2.2.0/icon_baker/trait.Icon.html) trait.

* `ico`
* `icns`
* _favicon_

### Icns Support

**Icon Baker** uses the [`icns`](https://crates.io/crates/icns) crate for generating `.icns` files. The 
supported [icon types](https://en.wikipedia.org/wiki/Apple_Icon_Image_format#Icon_types) are specified 
by the creators of such crate as follows:

| OSType | Description                                  | Supported?   |
|--------|----------------------------------------------|--------------|
| `ICON` | 32×32 1-bit entry                            | No           |
| `ICN#` | 32×32 1-bit entry with 1-bit mask            | No           |
| `icm#` | 16×12 1-bit entry with 1-bit mask            | No           |
| `icm4` | 16×12 4-bit entry                            | No           |
| `icm8` | 16×12 8-bit entry                            | No           |
| `ics#` | 16×16 1-bit mask                             | No           |
| `ics4` | 16×16 4-bit entry                            | No           |
| `ics8` | 16x16 8-bit entry                            | No           |
| `is32` | 16×16 24-bit entry                           | Yes          |
| `s8mk` | 16x16 8-bit mask                             | Yes          |
| `icl4` | 32×32 4-bit entry                            | No           |
| `icl8` | 32×32 8-bit entry                            | No           |
| `il32` | 32x32 24-bit entry                           | Yes          |
| `l8mk` | 32×32 8-bit mask                             | Yes          |
| `ich#` | 48×48 1-bit mask                             | No           |
| `ich4` | 48×48 4-bit entry                            | No           |
| `ich8` | 48×48 8-bit entry                            | No           |
| `ih32` | 48×48 24-bit entry                           | Yes          |
| `h8mk` | 48×48 8-bit mask                             | Yes          |
| `it32` | 128×128 24-bit entry                         | Yes          |
| `t8mk` | 128×128 8-bit mask                           | Yes          |
| `icp4` | 16x16 32-bit `png`/`jp2` entry               | `png` only   |
| `icp5` | 32x32 32-bit `png`/`jp2` entry               | `png` only   |
| `icp6` | 64x64 32-bit `png`/`jp2` entry               | `png` only   |
| `ic07` | 128x128 32-bit `png`/`jp2` entry             | `png` only   |
| `ic08` | 256×256 32-bit `png`/`jp2` entry             | `png` only   |
| `ic09` | 512×512 32-bit `png`/`jp2` entry             | `png` only   |
| `ic10` | 512x512@2x "retina" 32-bit `png`/`jp2` entry | `png` only   |
| `ic11` | 16x16@2x "retina" 32-bit `png`/`jp2` entry   | `png` only   |
| `ic12` | 32x32@2x "retina" 32-bit `png`/`jp2` entry   | `png` only   |
| `ic13` | 128x128@2x "retina" 32-bit `png`/`jp2` entry | `png` only   |
| `ic14` | 256x256@2x "retina" 32-bit `png`/`jp2` entry | `png` only   |

## Image Formats

**IconBaker** uses [`image`](https://crates.io/crates/image) for _raster graphics_ manipulations and 
[`resvg`](https://crates.io/crates/resvg/0.6.1) with the [`raqote`](https://crates.io/crates/raqote) 
backend for `svg` rasterization. Note that `raqote` requires [`cmake`](https://cmake.org/) to build.

| Format | Supported?                                                             | 
|--------|------------------------------------------------------------------------| 
| `png`  | All supported color types                                              | 
| `jpeg` | Baseline and progressive                                               | 
| `gif`  | Yes                                                                    | 
| `bmp`  | Yes                                                                    | 
| `ico`  | Yes                                                                    | 
| `tiff` | Baseline(no fax support), `lzw`, PackBits                              | 
| `webp` | Lossy(Luma channel only)                                               | 
| `pnm ` | `pbm`, `pgm`, `ppm`, standard `pma`                                    |
| `svg`  | [Static SVG Full 1.1](https://github.com/RazrFalcon/resvg#svg-support) |

## License

Licensed under MIT license([LICENSE-MIT](https://github.com/GarkGarcia/icon_baker/blob/master/LICENSE) 
or http://opensource.org/licenses/MIT).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the 
work by you shall be licensed as above, without any additional terms or conditions.

Feel free to help out! Contributions are welcomed 😃