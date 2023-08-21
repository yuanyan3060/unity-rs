use crate::pixel_info::{Pixel, SinglePixel};
use crate::ImageDecoder;
use byteorder::ReadBytesExt;
use std::io;

pub struct Alpha8;

impl ImageDecoder<1> for Alpha8 {
    const DECODE_PIXEL_BYTE: usize = 1;

    fn decode_pixel(data: &mut &[u8]) -> io::Result<SinglePixel> {
        Ok([Pixel::new_rgba(255, 255, 255, data.read_u8()?)])
    }
}
