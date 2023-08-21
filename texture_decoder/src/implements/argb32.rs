use crate::pixel_info::{Pixel, SinglePixel};
use crate::ImageDecoder;
use byteorder::ReadBytesExt;

pub struct ARGB32;

impl ImageDecoder for ARGB32 {
    const DECODE_PIXEL_BYTE: usize = 4;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        Ok([Pixel::builder().alpha(data.read_u8()?).rad(data.read_u8()?).green(data.read_u8()?).blue(data.read_u8()?).build()])
    }
}
