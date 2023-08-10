use byteorder::ReadBytesExt;
use crate::{ImageDecoder};

use crate::pixel_info::Pixel;

pub struct BGRA32;

impl ImageDecoder for BGRA32 {

    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(Pixel::builder().blue(data.read_u8()?).green(data.read_u8()?).rad(data.read_u8()?).alpha(data.read_u8()?).build())
    }
}
