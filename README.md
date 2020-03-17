# Ikon

[![Crate](https://img.shields.io/crates/v/ikon)](https://crates.io/crates/ikon)
[![API](https://docs.rs/ikon/badge.svg)](https://docs.rs/ikon)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.37+-lightgray.svg)
![Downloads](https://img.shields.io/crates/d/ikon)
[![License](https://img.shields.io/crates/l/ikon)](https://github.com/GarkGarcia/ikon/blob/master/LICENSE)

A robust, flexible framework for creating encoders and decoders for various 
_icon formats_.

# Overview

**Ikon** is intended to be used as a framework for developers interested 
in creating encoders and decoders for _various icon formats_ such as `.ico` 
files and _favicon_ schemes. It **does not** come with any encoders or 
decoders out of the box.

Instead, it simply automates much of the hard work of _encoding_, 
_decoding_ and _resampling_ different _image formats_, as well as provides 
powerfull abstractions, allowing developers to concentrate on the more
relevant problems.

_Icon families_ are represented as maps between _icons_ and _images_.The 
type of the _icons_ of an _icon_ is what determines how it can be 
indexed. 

## Icons

Each _icon format_ is associated with a particular type of _icon_. The type 
of the _icons_ of an _icon family_ is what determines how it can be indexed. 
Each _icon_ can only be associated with a single _image_.

## Resampling

Raster graphics are scaled using resampling filters, which are represented 
by _functions that take a source image and a size and return a re-scaled_ 
_image_.

This allows the users of `ikon` and any of it's dependant crates to provide 
their custom resampling filters. Common resampling filters are provided in 
the
[`resample`](https://docs.rs/ikon/0.1.0-beta.15/ikon/resample/index.html) 
module. The `resample` module also exposes the `resample::apply` function, 
which applies a resampling filter to an _image_ and checks if the outputted 
result matches the dimensions specified by the filter's arguments.

# Supported Image Formats

**Ikon** uses [`image`](https://crates.io/crates/image) for _raster graphics_ 
manipulations and [`resvg`](https://crates.io/crates/resvg) with the 
[`raqote`](https://crates.io/crates/raqote) backend for `svg` rasterization.

Note that some _file formats_ supported by `image` were explicitly left out of 
`ikon` because they were considered irrelevant to the library's domain.

| Format | Supported?                                                             | 
|--------|------------------------------------------------------------------------| 
| `png`  | All supported color types                                              | 
| `jpeg` | Baseline and progressive                                               | 
| `gif`  | Yes                                                                    | 
| `bmp`  | Yes                                                                    | 
| `webp` | Lossy(Luma channel only)                                               | 
| `svg`  | [Static SVG Full 1.1](https://github.com/RazrFalcon/resvg#svg-support) |

# Build Requirements

**Ikon** relies on [`harfbuzz_rs`](https://crates.io/crates/harfbuzz_rs), wich 
means [CMake](https://cmake.org/) is required to be installed for it build.

# License

Licensed under MIT license(
[LICENSE-MIT](https://github.com/GarkGarcia/ikon/blob/master/LICENSE) or 
http://opensource.org/licenses/MIT).

# Contribution

Unless you explicitly state otherwise, any contribution intentionally 
submitted for inclusion in the work by you shall be licensed as above, without 
any additional terms or conditions.

Feel free to help out! Contributions are welcomed 😃
