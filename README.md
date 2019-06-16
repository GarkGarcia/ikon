# IconBaker (LIB)
[![Crate](https://img.shields.io/crates/v/icon_baker.svg)](https://crates.io/crates/icon_baker)
[![API](https://docs.rs/icon_baker/badge.svg)](https://docs.rs/icon_baker)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.32+-lightgray.svg)](https://github.com/rust-random/rand#rust-version-requirements)

A simple solution for generating `.ico` and `.icns` icons. This crate serves as **IconBaker CLI's** internal library.

## Basic usage
```rust
use icon_baker::prelude::*;

const n_entrie: usize = 1;

fn main() {
    // Creating the icon
    let mut icon = Icon::ico(n_entries);

    // Importing the source image
    let src_image = SourceImage::from_file("img.jpg").unwrap();

    // Configuring the entry
    let opts = IconOptions::new(
        vec![(32, 32), (64, 64)] /* 32x32 and 64x64 icon */,
        ResamplingFilter::Linear /* Iterpolate the image */,
        Crop::Square             /* Square image */
    );

    // Adding the entry
    icon.add_entry(opts, &source_image).unwrap();
}

```

It is important to note that although the `Icon` returned by the `Icon::ico`, `Icon::icns`, `Icon::png_sequece` and `Icon::new` methods has the capacity specified, the `Icon` will have zero entries.

For an explanation of the difference between length and capacity, see
[*Capacity and reallocation*](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation).

## Writing to files
```rust
use icon_baker::prelude::*;
use std::fs::File;

const n_entrie: usize = ...;

fn main() {
    let mut icon = Icon::ico(n_entries);

    /* Process the icon */

    if let Ok(&file) = File::create("myfile.ico") {
        match icon.write(file) {
            Ok(()) => println!("File 'myfile.ico' saved!"),
            Err(_) => println!("An error occured ;-;")
        }
    }
}

```

## Limitations
There are two main limitations in this crate: both the `.icns` and `.svg` are not fully supported. Due to the use of external depencies, this crate is not able to fully support the formal specifications of those two file formats.

However, the coverage provided by these external dependencies should be more than enought for most use cases.

**Icon Baker** uses the `icns` crate for generating `.icns` files. The [supported icon types](https://github.com/mdsteele/rust-icns/blob/master/README.md#supported-icon-types) are specified by the creators of such crate as follows:

| OSType | Description                             | Supported? |
|--------|-----------------------------------------|------------|
| `ICON` | 32×32 1-bit icon                        | No         |
| `ICN#` | 32×32 1-bit icon with 1-bit mask        | No         |
| `icm#` | 16×12 1-bit icon with 1-bit mask        | No         |
| `icm4` | 16×12 4-bit icon                        | No         |
| `icm8` | 16×12 8-bit icon                        | No         |
| `ics#` | 16×16 1-bit mask                        | No         |
| `ics4` | 16×16 4-bit icon                        | No         |
| `ics8` | 16x16 8-bit icon                        | No         |
| `is32` | 16×16 24-bit icon                       | Yes        |
| `s8mk` | 16x16 8-bit mask                        | Yes        |
| `icl4` | 32×32 4-bit icon                        | No         |
| `icl8` | 32×32 8-bit icon                        | No         |
| `il32` | 32x32 24-bit icon                       | Yes        |
| `l8mk` | 32×32 8-bit mask                        | Yes        |
| `ich#` | 48×48 1-bit mask                        | No         |
| `ich4` | 48×48 4-bit icon                        | No         |
| `ich8` | 48×48 8-bit icon                        | No         |
| `ih32` | 48×48 24-bit icon                       | Yes        |
| `h8mk` | 48×48 8-bit mask                        | Yes        |
| `it32` | 128×128 24-bit icon                     | Yes        |
| `t8mk` | 128×128 8-bit mask                      | Yes        |
| `icp4` | 16x16 32-bit PNG/JP2 icon               | PNG only   |
| `icp5` | 32x32 32-bit PNG/JP2 icon               | PNG only   |
| `icp6` | 64x64 32-bit PNG/JP2 icon               | PNG only   |
| `ic07` | 128x128 32-bit PNG/JP2 icon             | PNG only   |
| `ic08` | 256×256 32-bit PNG/JP2 icon             | PNG only   |
| `ic09` | 512×512 32-bit PNG/JP2 icon             | PNG only   |
| `ic10` | 512x512@2x "retina" 32-bit PNG/JP2 icon | PNG only   |
| `ic11` | 16x16@2x "retina" 32-bit PNG/JP2 icon   | PNG only   |
| `ic12` | 32x32@2x "retina" 32-bit PNG/JP2 icon   | PNG only   |
| `ic13` | 128x128@2x "retina" 32-bit PNG/JP2 icon | PNG only   |
| `ic14` | 256x256@2x "retina" 32-bit PNG/JP2 icon | PNG only   |

On regards to SVG support, `icon_baker` uses the `nsvg` crate for rasterizing `.svg` files. According to the authors of the crate, _"`nsvg` does not provide all the functionality of NanoSVG yet. Just the bare minimum to create scaled rasters of SVGs. Like NanoSVG, the rasteriser only renders flat filled shapes. It is not particularly fast or accurate, but it is a simple way to bake vector graphics into textures"_.

The author of `icon_baker` is inclined to search for alternatives to `nsvg` if inquered to. Help would be appreciated. 