use crate::{ImageDecoder, ImageSize};
use byteorder::ReadBytesExt;
use bytes::BufMut;
use crate::pixel_info::Pixel;

pub struct ARGB32;

impl ImageDecoder for ARGB32 {

    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(
            Pixel::builder()
                .alpha(data.read_u8()?)
                .rad(data.read_u8()?)
                .green(data.read_u8()?)
                .blue(data.read_u8()?)
                .build()
        )
    }
}
