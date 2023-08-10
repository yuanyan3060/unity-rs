use crate::pixel_info::Pixel;
use crate::utils::clamp_byte;
use crate::{ImageDecoder, ImageSize};
use byteorder::ReadBytesExt;
use bytes::BufMut;

pub struct YUY2;

impl ImageDecoder for YUY2 {
    fn decoding(size: &ImageSize, mut img_data: &[u8], buffer: &mut impl BufMut) -> std::io::Result<()> {
        let data = &mut img_data;
        let half_width = size.width / 2;
        for _ in 0..size.height {
            for _ in 0..half_width {
                let (y0, u0) = (data.read_u8()? as u16, data.read_u8()? as u16);
                let (y1, v0) = (data.read_u8()? as u16, data.read_u8()? as u16);
                let c = y0 - 16;
                let d = u0 - 128;
                let e = v0 - 128;

                Pixel::builder()
                    .blue(clamp_byte((298 * c + 516 * d + 128) >> 8))
                    .green(clamp_byte((298 * c - 100 * d - 208 * e + 128) >> 8))
                    .rad(clamp_byte((298 * c + 409 * e + 128) >> 8))
                    .build()
                    .write_but(buffer);
                let c = y1 - 16;
                Pixel::builder()
                    .blue(clamp_byte((298 * c + 516 * d + 128) >> 8))
                    .green(clamp_byte((298 * c - 100 * d - 208 * e + 128) >> 8))
                    .rad(clamp_byte((298 * c + 409 * e + 128) >> 8))
                    .build()
                    .write_but(buffer);
            }
        }
        Ok(())
    }

    fn decode_step(_: &mut &[u8]) -> std::io::Result<Pixel> {
        Ok(Pixel::default())
    }
}
