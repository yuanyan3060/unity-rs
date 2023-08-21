use crate::pixel_info::{Pixel, SinglePixel};
use crate::utils::DownScaleToU8;
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};

pub struct R16;

impl ImageDecoder for R16 {
    const DECODE_PIXEL_BYTE: usize = 2;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        Ok([Pixel::builder().rad(data.read_u16::<BigEndian>()?.down_scale()).build()])
    }
}
