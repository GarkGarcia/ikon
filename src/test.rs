use crate::{icns::*, ico::*, png_sequence::*, resample, PathKey, Icon, SizeKey, SourceImage};
use image::{png::PNGEncoder, ColorType, GenericImageView};
use std::{fs::File, io::BufWriter};

macro_rules! png {
    ($r: expr, $s: expr, $w:expr) => {
        let scaled = $r(&$s, 32);
        let (w, h) = scaled.dimensions();
        let encoder = PNGEncoder::new($w);
        let data = scaled.to_rgba().into_raw();

        encoder
            .encode(&data, w, h, ColorType::RGBA(8))
            .expect("Could not encode or save the png output");
    };
}

#[test]
fn test_resample() {
    let mut file_near = File::create("tests/test_near.png").expect("Couldn't create file");

    let mut file_linear = File::create("tests/test_linear.png").expect("Couldn't create file");

    let mut file_cubic = File::create("tests/test_cubic.png").expect("Couldn't create file");

    let mut file_svg = File::create("tests/test_svg.png").expect("Couldn't create file");

    let hydra = SourceImage::open("tests/hydra.png").expect("File not found");

    let box_svg = SourceImage::open("tests/box.svg").expect("File not found");

    png!(resample::nearest, &hydra, &mut file_near);
    png!(resample::linear, &hydra, &mut file_linear);
    png!(resample::cubic, &hydra, &mut file_cubic);
    png!(resample::nearest, &box_svg, &mut file_svg);
}

#[test]
fn test_ico() {
    let mut file = BufWriter::new(File::create("tests/test.ico").expect("Couldn't create file"));

    let mut icon = Ico::new();
    let img = SourceImage::open("tests/hydra.png").expect("File not found");

    if let Err(err) = icon.add_entries(resample::nearest, &img, vec![SizeKey(32), SizeKey(64)]) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.add_entry(resample::nearest, &img, SizeKey(128)) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.add_entry(resample::nearest, &img, SizeKey(32)) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.write(&mut file) {
        panic!("{:?}", err);
    }
}

#[test]
fn test_icns() {
    let mut file = BufWriter::new(File::create("tests/test.icns").expect("Couldn't create file"));

    let mut icon = Icns::new();
    let img = SourceImage::open("tests/hydra.png").expect("File not found");

    if let Err(err) = icon.add_entries(resample::nearest, &img, vec![SizeKey(32), SizeKey(64)]) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.add_entry(resample::nearest, &img, SizeKey(128)) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.add_entry(resample::nearest, &img, SizeKey(32)) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.write(&mut file) {
        panic!("{:?}", err);
    }
}

#[test]
fn test_png() {
    let mut file = File::create("tests/test.tar").expect("Couldn't create file");

    let mut icon = PngSequence::new();
    let img = SourceImage::open("tests/hydra.png").expect("File not found");

    let entries = vec![
        PathKey::from(32, &"32/icon.png"),
        PathKey::from(64, &"64/icon.png"),
    ];

    if let Err(err) = icon.add_entries(resample::linear, &img, entries) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.write(&mut file) {
        panic!("{:?}", err);
    }
}
