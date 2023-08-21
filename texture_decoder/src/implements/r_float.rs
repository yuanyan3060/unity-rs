use byteorder::{BigEndian, ReadBytesExt};

use crate::pixel_info::{Pixel, SinglePixel};
use crate::utils::FloatConvU8;
use crate::ImageDecoder;

pub struct RFloat;

impl ImageDecoder for RFloat {
    const DECODE_PIXEL_BYTE: usize = 4;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        Ok(Pixel::builder().rad(data.read_f32::<BigEndian>()?.to_u8()).build().into())
    }
}
