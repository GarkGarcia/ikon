use std::io::{self, Write};
use nsvg::image::RgbaImage;
use super::{Result, Error};

pub fn png_sequence<W: Write>(bufs: Vec<RgbaImage>, w: W) -> Result<()> {
    let mut tar = tar::Builder::new(w);
    
    for buf in bufs {
        let size = buf.width();

        // Encode the pixel data as PNG and store it in a Vec<u8>
        let mut data = Vec::with_capacity(buf.len());
        if let Err(err) = png_encode_mini::write_rgba_from_u8(&mut data, &buf.clone().into_raw(), size, size) {
            return Err(Error::Io(err));
        }

        let file_name = format!("/{}.png", size);

        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_cksum();

        if let Err(err) = tar.append_data::<String, &[u8]>(&mut header, file_name, data.as_ref()) {
            return Err(Error::Io(err));
        }
    }

    Ok(())
}

pub fn ico<W: Write>(bufs: Vec<RgbaImage>, w: W) -> Result<()> {
    let mut output = ico::IconDir::new(ico::ResourceType::Icon);

    for buf in bufs {
        let size = buf.width();
        let data = ico::IconImage::from_rgba_data(size, size, buf.clone().into_vec());

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

pub fn icns<W: Write>(bufs: Vec<RgbaImage>, w: W) -> Result<()> {
    let mut output = icns::IconFamily::new();

    for buf in bufs {
        let size = buf.width();

        match icns::Image::from_data(icns::PixelFormat::RGBA, size, size, buf.clone().into_vec()) {
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