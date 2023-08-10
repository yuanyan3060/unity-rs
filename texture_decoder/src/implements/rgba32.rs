use crate::{ImageDecoder, ImageSize};
use byteorder::ReadBytesExt;
use bytes::BufMut;
use crate::pixel_info::Pixel;

pub struct RGBA32;

impl ImageDecoder for RGBA32 {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        let (r, g, b, a) = (data.read_u8()?, data.read_u8()?, data.read_u8()?, data.read_u8()?);
        let pixel = Pixel::new_rgba(r,g,b,a);
        Ok(pixel)
    }
}
