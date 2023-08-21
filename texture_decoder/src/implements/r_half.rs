use crate::pixel_info::{Pixel, SinglePixel};
use crate::utils::{FloatConvU8, ReadHalfFloat};
use crate::ImageDecoder;
use byteorder::BigEndian;

pub struct RHalf;

impl ImageDecoder for RHalf {
    const DECODE_PIXEL_BYTE: usize = 2;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        Ok(Pixel::builder().rad(data.read_f16::<BigEndian>()?.to_u8()).build().into())
    }
}
