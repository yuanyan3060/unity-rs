use crate::pixel_info::Pixel;
use crate::utils::DownScaleToU8;
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};

pub struct RGB48;

impl ImageDecoder for RGB48 {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(Pixel::builder()
            .rad(data.read_u16::<BigEndian>()?.down_scale())
            .green(data.read_u16::<BigEndian>()?.down_scale())
            .blue(data.read_u16::<BigEndian>()?.down_scale())
            .build())
    }
}
