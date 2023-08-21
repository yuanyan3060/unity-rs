use crate::pixel_info::{Pixel, SinglePixel};
use crate::ImageDecoder;
use byteorder::ReadBytesExt;

pub struct RGBA32;

impl ImageDecoder for RGBA32 {
    const DECODE_PIXEL_BYTE: usize = 4;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        let (r, g, b, a) = (data.read_u8()?, data.read_u8()?, data.read_u8()?, data.read_u8()?);
        let pixel = Pixel::new_rgba(r, g, b, a);
        Ok(pixel.into())
    }
}
