use crate::utils::down_scale_u16_to_u8;
use crate::{ImageDecoder, ImageSize};
use byteorder::{BigEndian, ReadBytesExt};
use bytes::BufMut;

pub struct R16;

impl ImageDecoder for R16 {
    fn decoding(size: &ImageSize, mut img_data: &[u8], buffer: &mut impl BufMut) -> std::io::Result<()> {
        let img = &mut img_data;
        for _ in 0..size.size() {
            buffer.put_u8(0);
            buffer.put_u8(0);
            buffer.put_u8(down_scale_u16_to_u8(img.read_u16::<BigEndian>()?));
            buffer.put_u8(255);
        }
        Ok(())
    }
}
