use crate::pixel_info::{Pixel, SinglePixel};
use crate::utils::DownScaleToU8;
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};

pub struct RG32;

impl ImageDecoder for RG32 {
    const DECODE_PIXEL_BYTE: usize = 4;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        Ok(Pixel::builder().rad(data.read_u16::<BigEndian>()?.down_scale()).green(data.read_u16::<BigEndian>()?.down_scale()).build().into())
    }
}
