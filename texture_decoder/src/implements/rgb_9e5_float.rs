use crate::pixel_info::Pixel;
use crate::utils::FloatConvU8;
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Error;

pub struct RGB9e5Float;

impl ImageDecoder for RGB9e5Float {
    const DECODE_PIXEL_BYTE: usize = 4;

    fn decode_pixel(data: &mut &[u8]) -> Result<[Pixel; 1], Error> {
        let val = data.read_i32::<BigEndian>()?;
        let scale = val >> 27 & 0x1f;
        let scale = 2f64.powf((scale - 24) as _);

        let b = (val >> 18 & 0x1ff) as f64;
        let g = (val >> 9 & 0x1ff) as f64;
        let r = (val & 0x1ff) as f64;

        Ok(Pixel::builder().rad((r * scale).to_u8()).green((g * scale).to_u8()).blue((b * scale).to_u8()).build().into())
    }
}
