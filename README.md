# Ikon

[![Crate](https://img.shields.io/crates/v/ikon.svg)](https://crates.io/crates/ikon)
[![API](https://docs.rs/ikon/badge.svg)](https://docs.rs/ikon)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.37+-lightgray.svg)
![Downloads](https://img.shields.io/crates/d/ikon)
[![License](https://img.shields.io/github/license/GarkGarcia/ikon)](https://github.com/GarkGarcia/ikon/blob/master/LICENSE)

A robust, flexible framework for creating encoders and decoders for various _icon formats_.

# Overview

An _icon_ consists of a map between _keys_ and _images_. An _entry_ is a _key-value_ pair contained
in an _icon_.

**Ikon** simply automates the process of re-scaling _images_, creating _entries_ and combining
them into an _icon_.

## Keys

Each _icon_ format is associated with a particular _key type_, which determines how
_entries_ are labeled. Each _key_ can only be associated with a single _image_.

For example, _icon_ formats that only differentiate _entries_ by the dimensions of their associated
_images_ are labeled by _positive integers_, such as the `.ico` and `.icns` file-formats.

On the other hand, _icon_ formats that distinguish their _entries_ by 
_[path](https://en.wikipedia.org/wiki/Path_%28computing%29)_, such as
_[FreeDesktop icon themes](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html)_
, are labeled by _path_.

Note that, since the dimensions of the _images_ contained in an _entry_ are dictated by their
associated _entries_, every _key_ must be convertible to a _positive integers_. Therefore, all
_key types_ are required to implement `AsSize`.

## Resampling

Pictures are scaled using resampling filters, which are represented by _functions that take a source_ 
_image and a size and return a re-scaled image_.

This allows the users of this crate to provide their custom resampling filters. Common resampling 
<<<<<<< HEAD
filters are provided in the 
[`resample`](https://docs.rs/icon_baker/2.2.0/icon_baker/resample/index.html) module.

# Examples

## General Usage

The `Icon::add_entry` can be used to automatically resample
_source images_ and converts them to _entries_ in an icon.

```rust
use icon_baker::{ico::{Ico, Key}, Image, Icon, IconError};

fn example() -> Result<(), IconError> {
    let icon = Ico::new();
    let src = Image::open("image.svg")?;

    icon.add_entry(resample::linear, &img, Key(32))
}
```

## Writing to Disk

Implementors of the `Icon` trait can be written to any object
that implements `io::Write` with the `Icon::write` method.

```rust
use icon_baker::{favicon::Favicon, Icon};
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
use icon_baker::{favicon::Favicon, Icon};
use std::{io, fs::File};
 
fn example() -> io::Result<()> {
    let icon = Favicon::new();

    /* Process the icon */

    icon.save("./output/")
}
```
=======
filters are provided in the [`resample`](https://docs.rs/ikon/2.2.0/ikon/resample/index.html) module.
>>>>>>> 4.0

# Support

**Ikon** uses [`image`](https://crates.io/crates/image) for _raster graphics_ manipulations and 
[`resvg`](https://crates.io/crates/resvg) with the [`raqote`](https://crates.io/crates/raqote) backend
for `svg` rasterization.

Note that some _file formats_ supported by `image` were explicitly left out of `ikon` because
they were considered irrelevant to the library's domain.

| Format | Supported?                                                             | 
|--------|------------------------------------------------------------------------| 
| `png`  | All supported color types                                              | 
| `jpeg` | Baseline and progressive                                               | 
| `gif`  | Yes                                                                    | 
| `bmp`  | Yes                                                                    | 
| `webp` | Lossy(Luma channel only)                                               | 
| `svg`  | [Static SVG Full 1.1](https://github.com/RazrFalcon/resvg#svg-support) |

# Build Requirements

**Ikon** relies on [`harfbuzz_rs`](https://crates.io/crates/harfbuzz_rs), wich means
[CMake](https://cmake.org/) is required to be installed for it build.

# License

Licensed under MIT license([LICENSE-MIT](https://github.com/GarkGarcia/ikon/blob/master/LICENSE) 
or http://opensource.org/licenses/MIT).

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the 
work by you shall be licensed as above, without any additional terms or conditions.

Feel free to help out! Contributions are welcomed 😃