use crate::pixel_info::{Pixel, SinglePixel};
use crate::ImageDecoder;
use byteorder::ReadBytesExt;

pub struct RG16;

impl ImageDecoder for RG16 {
    const DECODE_PIXEL_BYTE: usize = 2;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        Ok(Pixel::builder().rad(data.read_u8()?).green(data.read_u8()?).build().into())
    }
}
