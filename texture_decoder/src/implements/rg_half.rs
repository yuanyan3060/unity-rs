use byteorder::BigEndian;
use crate::{ImageDecoder};
use crate::pixel_info::Pixel;
use crate::utils::{FloatConvU8, ReadHalfFloat};

pub struct RGHalf;

impl ImageDecoder for RGHalf {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        let (r,g) = (data.read_f16::<BigEndian>()?.to_u8(),data.read_f16::<BigEndian>()?.to_u8());

        Ok(Pixel::builder().rad(r).green(g).build())
    }
}

