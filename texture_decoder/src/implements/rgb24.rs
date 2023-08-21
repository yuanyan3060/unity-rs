use crate::ImageDecoder;
use byteorder::ReadBytesExt;

use crate::pixel_info::{Pixel, SinglePixel};

pub struct RGB24;

impl ImageDecoder for RGB24 {
    const DECODE_PIXEL_BYTE: usize = 3;

    fn decode_pixel(iter: &mut &[u8]) -> std::io::Result<SinglePixel> {
        let (r, g, b) = (iter.read_u8()?, iter.read_u8()?, iter.read_u8()?);
        let pixel = Pixel::new_rgb(r, g, b);
        Ok(pixel.into())
    }
}
