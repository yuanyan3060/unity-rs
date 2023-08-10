use bytes::BufMut;
use crate::{ImageDecoder, ImageSize};

pub struct BGRA32;

impl ImageDecoder for BGRA32 {
    fn decoding(_size: &ImageSize, img_data: &[u8],buffer: &mut impl BufMut) -> std::io::Result<()> {
        buffer.put_slice(img_data);
        Ok(())
    }
}