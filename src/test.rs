use crate::{
    favicon::{self, Favicon},
    icns::{self, Icns},
    ico::{self, Ico},
    resample, Icon, SourceImage,
};
use image::{png::PNGEncoder, ColorType, DynamicImage, GenericImageView};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

fn png<F: FnMut(&SourceImage, u32) -> io::Result<DynamicImage>, W: Write>(
    mut filter: F,
    source: &SourceImage,
    w: W,
) -> io::Result<()> {
    let scaled = filter(source, 32)?;
    let (width, height) = scaled.dimensions();
    let encoder = PNGEncoder::new(w);
    let data = scaled.to_rgba().into_raw();

    encoder.encode(&data, width, height, ColorType::RGBA(8))
}

#[test]
fn test_resample() -> io::Result<()> {
    let mut file_near = File::create("tests/test_near.png").expect("Couldn't create file");
    let mut file_linear = File::create("tests/test_linear.png").expect("Couldn't create file");
    let mut file_cubic = File::create("tests/test_cubic.png").expect("Couldn't create file");
    let mut file_svg = File::create("tests/test_svg.png").expect("Couldn't create file");

    let hydra = SourceImage::open("tests/hydra.png").expect("File not found");
    let box_svg = SourceImage::open("tests/box.svg").expect("File not found");

    png(resample::nearest, &hydra, &mut file_near)?;
    png(resample::linear, &hydra, &mut file_linear)?;
    png(resample::cubic, &hydra, &mut file_cubic)?;
    png(resample::nearest, &box_svg, &mut file_svg)?;

    Ok(())
}

#[test]
fn test_ico() {
    let mut file = BufWriter::new(File::create("tests/test.ico").expect("Couldn't create file"));

    let mut icon = Ico::new();
    let img = SourceImage::open("tests/hydra.png").expect("File not found");

    let v = vec![ico::Key(32), ico::Key(64)];

    if let Err(err) = icon.add_entries(resample::nearest, &img, v) {
        panic!("{:?}", err);
    }

    // Should pass
    if let Err(err) = icon.add_entry(resample::nearest, &img, ico::Key(128)) {
        panic!("{:?}", err);
    }

    // Should fail
    if let Ok(_) = icon.add_entry(resample::nearest, &img, ico::Key(32)) {
        panic!("Should fail.");
    }

    if let Err(err) = icon.write(&mut file, &()) {
        panic!("{:?}", err);
    }
}

#[test]
fn test_icns() {
    let mut file = BufWriter::new(File::create("tests/test.icns").expect("Couldn't create file"));

    let mut icon = Icns::new();
    let img = SourceImage::open("tests/hydra.png").expect("Couldn't open file.");

    let entries = vec![icns::Key::Rgba32, icns::Key::Rgba64];
    if let Err(err) = icon.add_entries(resample::nearest, &img, entries) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.add_entry(resample::nearest, &img, icns::Key::Rgba128) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.write(&mut file, &()) {
        panic!("{:?}", err);
    }
}

#[test]
fn test_favicon() {
    let path = Path::new("tests/favicon/");

    let mut icon = Favicon::new();
    let hydra = SourceImage::open("tests/hydra.png").expect("Could not open `tests/hydra.png`.");
    let bbox = SourceImage::open("tests/box.svg").expect("Could not open `tests/box.svg`.");

    let entries = vec![favicon::Key(32), favicon::Key(64)];

    if let Err(err) = icon.add_entry(resample::nearest, &hydra, favicon::Key(16)) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.add_entries(resample::cubic, &bbox, entries) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.save(&path, &favicon::WriteOptions::default()) {
        panic!("{:?}", err);
    }
}

/*fn test_png() {
    let mut file = File::create("tests/test.tar").expect("Couldn't create file");

    let mut icon = PngSequence::new();
    let img = SourceImage::open("tests/hydra.png").expect("File not found");

    let entries = vec![
        PngKey::from(32, "32/icon.png").unwrap(),
        PngKey::from(64, "64/icon.png").unwrap(),
    ];

    if let Err(err) = icon.add_entries(resample::linear, &img, entries) {
        panic!("{:?}", err);
    }

    if let Err(err) = icon.write(&mut file) {
        panic!("{:?}", err);
    }
} */
