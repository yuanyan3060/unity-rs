use crate::{ImageDecoder, ImageSize};
use bytes::BufMut;
use std::io;
use byteorder::ReadBytesExt;
use crate::pixel_info::Pixel;

pub struct Alpha8;

impl ImageDecoder for Alpha8 {
    fn decode_step(data: &mut &[u8]) -> io::Result<Pixel> {
        Ok(Pixel::new_rgba(255, 255, 255, data.read_u8()?))
    }
}
