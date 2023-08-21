use crate::ImageDecoder;
use byteorder::ReadBytesExt;

use crate::pixel_info::{Pixel, SinglePixel};

pub struct BGRA32;

impl ImageDecoder for BGRA32 {
    const DECODE_PIXEL_BYTE: usize = 4;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        Ok([Pixel::builder().blue(data.read_u8()?).green(data.read_u8()?).rad(data.read_u8()?).alpha(data.read_u8()?).build()])
    }
}
