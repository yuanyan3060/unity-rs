use byteorder::ReadBytesExt;
use bytes::BufMut;
use crate::{ImageDecoder, ImageSize};

pub struct RGBA32;

impl ImageDecoder for RGBA32 {
    fn decoding(size: &ImageSize, mut img_data: &[u8], buffer: &mut impl BufMut) -> std::io::Result<()> {
        let data = &mut img_data;
        for _ in 0..size.size(){

            let (r,g,b,a) = (data.read_u8()?,data.read_u8()?,data.read_u8()?,data.read_u8()?);
            buffer.put_u8(b);
            buffer.put_u8(g);
            buffer.put_u8(r);
            buffer.put_u8(a);
        }
        Ok(())
    }
}