use crate::utils::down_scale_u16_to_u8;
use crate::{ImageDecoder, ImageSize};
use byteorder::{BigEndian, ReadBytesExt};
use bytes::BufMut;
use crate::pixel_info::Pixel;

pub struct R16;

impl ImageDecoder for R16 {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(Pixel::builder().rad(down_scale_u16_to_u8(data.read_u16::<BigEndian>()?)).build())
    }
}
