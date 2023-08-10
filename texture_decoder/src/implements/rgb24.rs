use crate::ImageDecoder;
use byteorder::ReadBytesExt;

use crate::pixel_info::Pixel;

pub struct RGB24;

impl ImageDecoder for RGB24 {
    fn decode_step(iter: &mut &[u8]) -> std::io::Result<Pixel> {
        let (r, g, b) = (iter.read_u8()?, iter.read_u8()?, iter.read_u8()?);
        let pixel = Pixel::new_rgb(r, g, b);
        Ok(pixel)
    }
}
