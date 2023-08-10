use crate::{ImageDecoder, ImageSize};
use byteorder::ReadBytesExt;
use bytes::BufMut;

pub struct ARGB32;

impl ImageDecoder for ARGB32 {
    fn decoding(size: &ImageSize, mut img_data: &[u8], buffer: &mut impl BufMut) -> std::io::Result<()> {
        let data = &mut img_data;
        for _ in 0..size.size() {
            let (a, r, g, b) = (data.read_u8()?, data.read_u8()?, data.read_u8()?, data.read_u8()?);
            buffer.put_u8(b);
            buffer.put_u8(g);
            buffer.put_u8(r);
            buffer.put_u8(a);
        }
        Ok(())
    }
}
