use crate::pixel_info::Pixel;
use crate::utils::FloatConvU8;
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Error;

pub struct RGFloat;

impl ImageDecoder for RGFloat {
    const DECODE_PIXEL_BYTE: usize = 8;

    fn decode_pixel(data: &mut &[u8]) -> Result<[Pixel; 1], Error> {
        Ok(Pixel::builder().rad(data.read_f32::<BigEndian>()?.to_u8()).green(data.read_f32::<BigEndian>()?.to_u8()).build().into())
    }
}
