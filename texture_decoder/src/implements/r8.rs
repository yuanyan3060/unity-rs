use byteorder::ReadBytesExt;
use crate::ImageDecoder;
use crate::pixel_info::Pixel;

pub struct R8;

impl ImageDecoder for R8 {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(Pixel::builder().rad(data.read_u8()?).build())
    }
}

