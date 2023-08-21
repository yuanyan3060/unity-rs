use crate::pixel_info::{Pixel, SinglePixel};
use crate::utils::FloatConvU8;
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};

pub struct RGBAFloat;

impl ImageDecoder for RGBAFloat {
    const DECODE_PIXEL_BYTE: usize = 16;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        Ok(Pixel::builder()
            .rad(data.read_f32::<BigEndian>()?.to_u8())
            .green(data.read_f32::<BigEndian>()?.to_u8())
            .blue(data.read_f32::<BigEndian>()?.to_u8())
            .alpha(data.read_f32::<BigEndian>()?.to_u8())
            .build()
            .into())
    }
}
