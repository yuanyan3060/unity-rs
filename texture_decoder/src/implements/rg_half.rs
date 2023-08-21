use crate::pixel_info::{Pixel, SinglePixel};
use crate::utils::{FloatConvU8, ReadHalfFloat};
use crate::ImageDecoder;
use byteorder::BigEndian;

pub struct RGHalf;

impl ImageDecoder for RGHalf {
    const DECODE_PIXEL_BYTE: usize = 4;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        let (r, g) = (data.read_f16::<BigEndian>()?.to_u8(), data.read_f16::<BigEndian>()?.to_u8());

        Ok(Pixel::builder().rad(r).green(g).build().into())
    }
}
