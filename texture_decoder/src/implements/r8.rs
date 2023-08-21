use crate::pixel_info::{Pixel, SinglePixel};
use crate::ImageDecoder;
use byteorder::ReadBytesExt;

pub struct R8;

impl ImageDecoder for R8 {
    const DECODE_PIXEL_BYTE: usize = 1;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        Ok([Pixel::builder().rad(data.read_u8()?).build()])
    }
}
