# IconBaker

[![Crate](https://img.shields.io/crates/v/icon_baker.svg)](https://crates.io/crates/icon_baker)
[![API](https://docs.rs/icon_baker/badge.svg)](https://docs.rs/icon_baker)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.37+-lightgray.svg)
![Downloads](https://img.shields.io/crates/d/icon_baker)
[![License](https://img.shields.io/github/license/GarkGarcia/icon_baker)](https://github.com/GarkGarcia/icon_baker/blob/master/LICENSE)

A simple solution for encoding common icon file-formats, such as `.ico` and `.icns`. 

This crate is mostly a wrapper for other libraries, unifying existing APIs into a single, cohesive 
interface. It serves as **[IconPie's](https://github.com/GarkGarcia/icon-pie)** internal library.

# Overview

An _icon_ consists of a set of _entries_. An _entry_ is simply an image that has a particular size.
**IconBaker** simply automates the process of re-scaling pictures and combining them into an _icon_.

Pictures are scaled using resampling filters, which are represented by _functions that take a source_ 
_image and a size and return a re-scaled image_.

This allows the users of this crate to provide their custom resampling filters. Common resampling 
filters are provided in the 
[`resample`](https://docs.rs/icon_baker/2.2.0/icon_baker/resample/index.html) module.

# Examples

## General Usage

```rust
use icon_baker::{Ico, SourceImage, Icon};
use icon_baker::Error as IconError;
 
fn example() -> Result<(), IconError> {
    let icon = Ico::new();

    match SourceImage::from_path("image.svg") {
        Some(img) => icon.add_entry(resample::linear, &img, 32),
        None      => Ok(())
    }
}
```

## Writing to a File

```rust
use icon_baker::*;
use std::{io, fs::File};
 
fn example() -> io::Result<()> {
    let icon = PngSequence::new();

    /* Process the icon */

    let file = File::create("out.icns")?;
    icon.write(file)
}
```

# Support

## Icon Formats

This are the output formats **IconBaker** nativally supports. Be aware that custum output types can 
be created using the [`Icon`](https://docs.rs/icon_baker/2.2.0/icon_baker/trait.Icon.html) trait.

* `ico`
* `icns`
* `png` sequence (`tar`)

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
