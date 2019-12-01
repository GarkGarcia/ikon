use crate::{encode, resample, Image};
use std::{
    fs::File,
    io::{self, Write},
};

#[test]
fn load() -> io::Result<()> {
    if let Err(err) = Image::load(File::open("tests/test.png")?) {
        panic!("FAILED AT PNG {:?}", err);
    }

    if let Err(err) = Image::load(File::open("tests/test.jpg")?) {
        panic!("FAILED AT JPG {:?}", err);
    }

    if let Err(err) = Image::load(File::open("tests/test.gif")?) {
        panic!("FAILED AT GIF {:?}", err);
    }

    if let Err(err) = Image::load(File::open("tests/test.bmp")?) {
        panic!("FAILED AT BMP {:?}", err);
    }

    if let Err(err) = Image::load(File::open("tests/test.webp")?) {
        panic!("FAILED AT WEBP {:?}", err);
    }

    if let Err(err) = Image::load(File::open("tests/test.svg")?) {
        panic!("FAILED AT SVG {:?}", err);
    }

    Ok(())
}

#[test]
fn rasterize() -> io::Result<()> {
    let mut file_near = File::create("tests/rasterize/near.png").expect("Couldn't create file");
    let mut file_linear = File::create("tests/rasterize/linear.png").expect("Couldn't create file");
    let mut file_cubic = File::create("tests/rasterize/cubic.png").expect("Couldn't create file");
    let mut file_svg = File::create("tests/rasterize/svg.png").expect("Couldn't create file");

    let source_png = Image::open("tests/test.png").expect("File not found");
    let source_svg = Image::open("tests/test.svg").expect("File not found");

    let buf = encode::png(&source_png.rasterize(resample::nearest, (32, 32)).expect("Failed"))?;
    file_near.write_all(buf.as_ref())?;

    let buf = encode::png(&source_png.rasterize(resample::linear, (32, 32)).expect("Failed"))?;
    file_linear.write_all(buf.as_ref())?;

    let buf = encode::png(&source_png.rasterize(resample::cubic, (32, 32)).expect("Failed"))?;
    file_cubic.write_all(buf.as_ref())?;

    let buf = encode::png(&source_svg.rasterize(resample::nearest, (32, 32)).expect("Failed"))?;
    file_svg.write_all(buf.as_ref())?;

    Ok(())
}
