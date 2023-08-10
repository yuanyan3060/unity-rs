use crate::pixel_info::Pixel;
use crate::utils::DownScaleToU8;
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};

pub struct RG32;

impl ImageDecoder for RG32 {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(Pixel::builder().rad(data.read_u16::<BigEndian>()?.down_scale()).green(data.read_u16::<BigEndian>()?.down_scale()).build())
    }
}
