use crate::pixel_info::Pixel;
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};

pub struct RGB565;

impl ImageDecoder for RGB565 {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        let p = data.read_u16::<BigEndian>()?;
        let pixel = Pixel::builder().blue(((p << 2) | (p >> 3 & 7)) as _).green(((p >> 3 & 0xfc) | p >> 9 & 3) as _).rad(((p >> 8 & 0xf8) | (p >> 13)) as _).build();
        Ok(pixel)
    }
}
