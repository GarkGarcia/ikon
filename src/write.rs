use std::io::{self, Write, Seek};
use nsvg::image::RgbaImage;
use zip::result::ZipError;
use super::{Result, Error};

pub fn png_sequence<W: Write + Seek>(bufs: Vec<RgbaImage>, w: W) -> Result<()> {
    let mut zip = zip::ZipWriter::new(w);
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    
    for buf in bufs {
        let (w, h) = buf.dimensions();

        // Encode the pixel data as PNG and store it in a Vec<u8>
        let mut data = Vec::with_capacity(buf.len());
        if let Err(err) = png_encode_mini::write_rgba_from_u8(&mut data, &buf.clone().into_raw(), w, h) {
            return Err(Error::Io(err));
        }

        let file_name = if w == h {
            format!("{}.png", w)
        } else {
            format!("{}x{}.png", w, h)
        };

        if let Err(err) = zip.start_file(file_name, options) {
            match err {
                ZipError::Io(err) => return Err(Error::Io(err)),
                _ => return Err(Error::Zip(err))
            }
        }

        if let Err(err) = zip.write_all(&data[..]) {
            return Err(Error::Io(err))
        }
    }

    if let Err(err) = zip.finish() {
        match err {
            ZipError::Io(err) => return Err(Error::Io(err)),
            _ => return Err(Error::Zip(err))
        }
    } else {
        Ok(())
    }
}

pub fn ico<W: Write + Seek>(bufs: Vec<RgbaImage>, w: W) -> Result<()> {
    let mut output = ico::IconDir::new(ico::ResourceType::Icon);

    for buf in bufs {
        let (w, h) = buf.dimensions();
        let data = ico::IconImage::from_rgba_data(w, h, buf.clone().into_vec());

        match ico::IconDirEntry::encode(&data) {
            Ok(entry) => output.add_entry(entry),
            Err(err) => return Err(Error::Io(err))
        }
    }

    match output.write(w) {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::Io(err))
    }
}

pub fn icns<W: Write + Seek>(bufs: Vec<RgbaImage>, w: W) -> Result<()> {
    let mut output = icns::IconFamily::new();

    for buf in bufs {
        let (w, h) = buf.dimensions();

        match icns::Image::from_data(icns::PixelFormat::RGBA, w, h, buf.clone().into_vec()) {
            Ok(icon) => if let Err(err) = output.add_icon(&icon) {
                return Err(Error::Io(err))
            },
            Err(err) => return Err(Error::Io(err))
        }
    }

    let buf_writer = io::BufWriter::new(w);
    match output.write(buf_writer) {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::Io(err))
    }
}