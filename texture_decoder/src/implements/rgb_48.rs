use crate::pixel_info::Pixel;
use crate::utils::DownScaleToU8;
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};

pub struct RGB48;

impl ImageDecoder for RGB48 {
    const DECODE_PIXEL_BYTE: usize = 6;

    fn decode_pixel(data: &mut &[u8]) -> Result<[Pixel; 1], std::io::Error> {
        Ok(Pixel::builder()
            .rad(data.read_u16::<BigEndian>()?.down_scale())
            .green(data.read_u16::<BigEndian>()?.down_scale())
            .blue(data.read_u16::<BigEndian>()?.down_scale())
            .build()
            .into())
    }
}
