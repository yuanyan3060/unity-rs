use crate::pixel_info::Pixel;
use crate::utils::{FloatConvU8, ReadHalfFloat};
use crate::ImageDecoder;
use byteorder::BigEndian;

pub struct RGBAFloat;

impl ImageDecoder for RGBAFloat {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(Pixel::builder()
            .rad(data.read_f16::<BigEndian>()?.to_u8())
            .green(data.read_f16::<BigEndian>()?.to_u8())
            .blue(data.read_f16::<BigEndian>()?.to_u8())
            .alpha(data.read_f16::<BigEndian>()?.to_u8())
            .build())
    }
}
