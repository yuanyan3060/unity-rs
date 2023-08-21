use crate::pixel_info::{Pixel, SinglePixel};
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};

pub struct RGB565;

impl ImageDecoder for RGB565 {
    const DECODE_PIXEL_BYTE: usize = 2;

    fn decode_pixel(data: &mut &[u8]) -> std::io::Result<SinglePixel> {
        let p = data.read_u16::<BigEndian>()?;
        let pixel = Pixel::builder().blue(((p << 2) | (p >> 3 & 7)) as _).green(((p >> 3 & 0xfc) | p >> 9 & 3) as _).rad(((p >> 8 & 0xf8) | (p >> 13)) as _).build();
        Ok(pixel.into())
    }
}
