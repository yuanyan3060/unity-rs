use crate::pixel_info::Pixel;
use crate::ImageDecoder;
use byteorder::{BigEndian, ReadBytesExt};

pub struct RGBA4444;

impl ImageDecoder for RGBA4444 {
    fn decode_step(data: &mut &[u8]) -> std::io::Result<Pixel> {
        let mut pixel_buff = [0u8; 4];
        let pixel_old = data.read_u16::<BigEndian>()?;
        pixel_buff[0] = ((pixel_old & 0x00f0) >> 4) as _;
        pixel_buff[1] = ((pixel_old & 0x0f00) >> 8) as _;
        pixel_buff[2] = ((pixel_old & 0xf000) >> 12) as _;
        pixel_buff[3] = (pixel_old & 0x000f) as _;

        for pixel in pixel_buff.iter_mut() {
            *pixel = (*pixel << 4) | *pixel
        }
        let [b, g, r, a] = pixel_buff;

        Ok(Pixel::new_rgba(r, g, b, a))
    }
}
