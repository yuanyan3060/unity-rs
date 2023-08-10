use crate::pixel_info::Pixel;
use crate::utils::{FloatConvU8, ReadHalfFloat};
use crate::ImageDecoder;
use byteorder::BigEndian;

pub struct RGBAHalf;

impl ImageDecoder for RGBAHalf {
    fn decode_step(img: &mut &[u8]) -> std::io::Result<Pixel> {
        let (r, g, b, a) = (img.read_f16::<BigEndian>()?.to_u8(), img.read_f16::<BigEndian>()?.to_u8(), img.read_f16::<BigEndian>()?.to_u8(), img.read_f16::<BigEndian>()?.to_u8());

        let pixel = Pixel::new_rgba(r, g, b, a);
        Ok(pixel)
    }
}
