use crate::pixel_info::Pixel;
use crate::ImageDecoder;
use byteorder::ReadBytesExt;
use std::io;

pub struct Alpha8;

impl ImageDecoder for Alpha8 {
    fn decode_step(data: &mut &[u8]) -> io::Result<Pixel> {
        Ok(Pixel::new_rgba(255, 255, 255, data.read_u8()?))
    }
}
