use crate::{ImageDecoder, ImageSize};
use byteorder::{BigEndian, ReadBytesExt};
use bytes::BufMut;

pub struct RGB565;

impl ImageDecoder for RGB565 {
    fn decoding(size: &ImageSize, mut img_data: &[u8], buffer: &mut impl BufMut) -> std::io::Result<()> {
        let data = &mut img_data;
        for _ in 0..size.size() {
            let p = data.read_u16::<BigEndian>()?;

            buffer.put_u8(((p << 2) | (p >> 3 & 7)) as _);
            buffer.put_u8(((p >> 3 & 0xfc) | p >> 9 & 3) as _);
            buffer.put_u8(((p >> 8 & 0xf8) | (p >> 13)) as _);
            buffer.put_u8(255)
        }
        Ok(())
    }
}
