# IconBaker (LIB)
[![Crate](https://img.shields.io/crates/v/icon_baker.svg)](https://crates.io/crates/icon_baker)
[![API](https://docs.rs/icon_baker/badge.svg)](https://docs.rs/icon_baker)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.32+-lightgray.svg)](https://github.com/rust-random/rand#rust-version-requirements)

A simple solution for generating `.ico` and `.icns` icons. This crate serves as **IconBaker CLI's** internal library.

## Usage
This crate's API revolves around the concept of binding source images to a set of sizes and resampling configurations. The following example demonstrates this principle:

```rust
use icon_baker::prelude::*;

const N_ENTRIES: usize = 1;

fn main() {
    // Creating the icon
    let mut icon = Icon::ico(n_entries);

    // Importing the source image
    let src_image = SourceImage::from_file("img.jpg").unwrap();

    // Configuring the entry
    let opts = IconOptions::new(
        vec![(32, 32), (64, 64)] /* 32x32 and 64x64 sizes */,
        ResamplingFilter::Linear /* Iterpolate the image */,
        Crop::Square             /* Square image */
    );

    // Adding the entry
    icon.add_entry(opts, &src_image).unwrap();
}
```

Note that the `capacity` argument in the `Icon::ico`, `Icon::icns`, `Icon::png_sequence` and `Icon::new` methods specifies the expected number of _entries_ in a given `Icon`, **_NOT_** the number of _sizes_ (since a single entry can contain multiple sizes).

### Sampling From Multiple Sources
Let's say you want to customize your icon so that the smaller versions of it are less detailed. **IconBaker** helps you achieve this by allowing to sample from multiple sources.

You can simply combine separate source images by specifying to which entry they should be assigned:

```rust
use icon_baker::prelude::*;

const N_ENTRIES: usize = 2;

fn main() {
    let option = IconOptions::new;
    let mut icon = Icon::ico(n_entries);

    // Importing the source images
    let small = SourceImage::from_file("small.jpg").unwrap();
    let large = SourceImage::from_file("small.png").unwrap();

    // Configuring the entries
    let filter = ResamplingFilter::Nearest;
    let crop = Crop::Square;

    let small_opt = option(vec![(16, 16)], filter, crop);
    let large_opt = option(vec![(32, 32)], filter, crop);

    // Adding the entries
    icon.add_entry(small_opt, &small).unwrap();
    icon.add_entry(large_opt, &large).unwrap();
}
```

Note that different `IconOptions` instances do not need to share the same rasampling options (namely the `filter` and `crop` fields) and can even share a common `source` field.

However, different entries cannot share a common `Size` in their `sizes` fields. For example, the following program will panic at second call of `icon.add_entry()`:

```rust
use icon_baker::prelude::*;

const N_ENTRIES: usize = 2;

fn main() {
    let option = IconOptions::new;
    let mut icon = Icon::ico(n_entries);

    // Importing the source images
    let src1 = SourceImage::from_file("src1.jpg").unwrap();
    let src2 = SourceImage::from_file("src2.png").unwrap();

    // Configuring the entries
    let filter = ResamplingFilter::Nearest;
    let crop = Crop::Square;

    let src1_opt = option(vec![(16, 16)], filter, crop);
    let src2_opt = option(vec![(16, 16)], filter, crop);

    // Adding the entries
    icon.add_entry(src1_opt, &src1).expect("Returns Ok(())");
    icon.add_entry(src2_opt, &src2).expect(
        "Returns Err(icon_baker::Error::SizeAlreadyIncluded((16, 16))))");
}
```

### Writing to Files
Writing to files can be easily done by calling the `Icon::write` method:

```rust
use icon_baker::prelude::*;
use std::fs::File;

/* Const declarations */

fn main() {
    let mut icon = Icon::ico(N_ENTRIES);

    /* Process the icon */

    if let Ok(&file) = File::create("myfile.ico") {
        match icon.write(file) {
            Ok(()) => println!("File 'myfile.ico' saved!"),
            Err(_) => println!("An error occurred ;-;")
        }
    }
}
```

Note that the `Icon::write` method can also write to instances of any type which implements [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html).

## Supported Image Formats
| Format | Supported?                                         | 
| ------ | -------------------------------------------------- | 
| `PNG`  | All supported color types                          | 
| `JPEG` | Baseline and progressive                           | 
| `GIF`  | Yes                                                | 
| `BMP`  | Yes                                                | 
| `ICO`  | Yes                                                | 
| `TIFF` | Baseline(no fax support), `LZW`, PackBits          | 
| `WEBP` | Lossy(Luma channel only)                           | 
| `PNM ` | `PBM`, `PGM`, `PPM`, standard `PAM`                |
| `SVG`  | Limited([flat filled shapes only](#svg-support))   |

## Limitations
There are two main limitations in this crate: both `ICNS` and `SVG` are not fully supported. Due to the use of external dependencies, this crate is not able to fully support the formal specifications of those two file formats.

However, the coverage provided by this external dependencies should be enough for most use cases.

### ICNS Support

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

### SVG Support

**IconBaker** uses the `nsvg` crate for rasterizing `.svg` files. According to the authors of the crate:

> Like NanoSVG, the rasterizer only renders flat filled shapes. It is not particularly fast or accurate, but it is a simple way to bake vector graphics into textures.

The author of `icon_baker` is inclined to search for alternatives to `nsvg` if inquired to. Help would be appreciated. 