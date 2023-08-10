use byteorder::{BigEndian, ReadBytesExt};
use crate::ImageDecoder;
use crate::pixel_info::Pixel;
use crate::utils::FloatConvU8;

pub struct RGB9e5Float;

impl ImageDecoder for RGB9e5Float {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        let val = data.read_i32::<BigEndian>()?;
        let scale = val>>27 &0x1f;
        let scale = 2f64.powf((scale-24) as _);

        let b = (val >> 18 & 0x1ff) as f64;
        let g = (val >> 9 & 0x1ff) as f64;
        let r = (val  & 0x1ff) as f64;

        Ok(Pixel::builder()
            .rad((r*scale).to_u8())
            .green((g*scale).to_u8())
            .blue((b*scale).to_u8())
            .build()
        )
    }
}
