use crate::{encode, resample, Image};
use std::{
    fs::File,
    io::{self, Write},
};

#[test]
fn test_load() -> io::Result<()> {
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

    // if let Err(err) = Image::load(File::open("tests/test.webp")?) {
    //     panic!("FAILED AT WEBP {:?}", err);
    // }

    if let Err(err) = Image::load(File::open("tests/test.svg")?) {
        panic!("FAILED AT SVG {:?}", err);
    }

    Ok(())
}

#[test]
fn test_resample() -> io::Result<()> {
    let mut file_near = File::create("tests/test_near.png").expect("Couldn't create file");
    let mut file_linear = File::create("tests/test_linear.png").expect("Couldn't create file");
    let mut file_cubic = File::create("tests/test_cubic.png").expect("Couldn't create file");
    let mut file_svg = File::create("tests/test_svg.png").expect("Couldn't create file");

    let hydra = Image::open("tests/test.png").expect("File not found");
    let box_svg = Image::open("tests/test.svg").expect("File not found");

    let buf = encode::png(&hydra.rasterize(resample::nearest, 32).expect("Failed"))?;
    file_near.write_all(buf.as_ref())?;

    let buf = encode::png(&hydra.rasterize(resample::linear, 32).expect("Failed"))?;
    file_linear.write_all(buf.as_ref())?;

    let buf = encode::png(&hydra.rasterize(resample::cubic, 32).expect("Failed"))?;
    file_cubic.write_all(buf.as_ref())?;

    let buf = encode::png(&box_svg.rasterize(resample::nearest, 32).expect("Failed"))?;
    file_svg.write_all(buf.as_ref())?;

    Ok(())
}
