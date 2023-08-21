use crate::pixel_info::Pixel;
use crate::utils::clamp_byte;
use crate::ImageDecoder;
use byteorder::ReadBytesExt;
use std::io;

pub struct YUY2;

impl ImageDecoder<2> for YUY2 {
    const DECODE_PIXEL_BYTE: usize = 4;

    fn decode_pixel(data: &mut &[u8]) -> Result<[Pixel; 2], io::Error> {
        let (y0, u0) = (data.read_u8()? as u16, data.read_u8()? as u16);
        let (y1, v0) = (data.read_u8()? as u16, data.read_u8()? as u16);
        let c = y0 - 16;
        let d = u0 - 128;
        let e = v0 - 128;

        let pix1 = yuy2_pixel(c, d, e);
        let c = y1 - 16;
        let pix2 = yuy2_pixel(c, d, e);

        Ok([pix1, pix2])
    }
}

fn yuy2_pixel(c: u16, d: u16, e: u16) -> Pixel {
    Pixel::builder()
        .blue(clamp_byte((298 * c + 516 * d + 128) >> 8))
        .green(clamp_byte((298 * c - 100 * d - 208 * e + 128) >> 8))
        .rad(clamp_byte((298 * c + 409 * e + 128) >> 8))
        .build()
}
