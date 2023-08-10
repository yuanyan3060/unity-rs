use byteorder::ReadBytesExt;
use bytes::{BufMut, BytesMut};
use crate::{ImageDecoder, ImageSize};

pub struct RGB24;

impl ImageDecoder for RGB24 {
    fn decoding(size: &ImageSize,mut img_data: &[u8], buffer: &mut impl BufMut) -> std::io::Result<()> {
        let size = size.size();
        let iter = &mut img_data;
        for _ in 0..size{
            let(r,g,b) = (iter.read_u8()?,iter.read_u8()?,iter.read_u8()?);

            buffer.put_u8(b);
            buffer.put_u8(g);
            buffer.put_u8(r);
            buffer.put_u8(255);
        }
        Ok(())
    }
}
