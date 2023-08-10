use byteorder::{BigEndian, ReadBytesExt};

use crate::{ImageDecoder};
use crate::pixel_info::Pixel;
use crate::utils::FloatConvU8;

pub struct RFloat;

impl ImageDecoder for RFloat {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(Pixel::builder().rad(data.read_f32::<BigEndian>()?.to_u8()).build())
    }
}