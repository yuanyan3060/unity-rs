use crate::pixel_info::Pixel;
use crate::utils::{FloatConvU8, ReadHalfFloat};
use crate::ImageDecoder;
use byteorder::BigEndian;

pub struct RHalf;

impl ImageDecoder for RHalf {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(Pixel::builder().rad(data.read_f16::<BigEndian>()?.to_u8()).build())
    }
}
