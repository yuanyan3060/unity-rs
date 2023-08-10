use byteorder::{BigEndian, ReadBytesExt};
use crate::ImageDecoder;
use crate::pixel_info::Pixel;
use crate::utils::DownScaleToU8;

pub struct RGBA64;

impl ImageDecoder for RGBA64 {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(Pixel::builder()
            .rad(data.read_u16::<BigEndian>()?.down_scale())
            .green(data.read_u16::<BigEndian>()?.down_scale())
            .blue(data.read_u16::<BigEndian>()?.down_scale())
            .alpha(data.read_u16::<BigEndian>()?.down_scale())
            .build())
    }
}