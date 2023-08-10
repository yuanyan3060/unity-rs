use crate::{ImageDecoder, ImageSize};
use byteorder::{BigEndian, ReadBytesExt};
use bytes::BufMut;
use std::io;

pub struct ARGB4444;

impl ImageDecoder for ARGB4444 {
    fn decoding(size: &ImageSize, mut img_data: &[u8], buffer: &mut impl BufMut) -> io::Result<()> {
        let size = size.size();
        let mut pixel_buff = [0u8; 4];
        let img = &mut img_data;
        for _ in 0..size {
            let pixel_old = img.read_u16::<BigEndian>()?;
            pixel_buff[0] = (pixel_old & 0x000f) as _;
            pixel_buff[1] = ((pixel_old & 0x00f0) >> 4) as _;
            pixel_buff[2] = ((pixel_old & 0x0f00) >> 8) as _;
            pixel_buff[3] = ((pixel_old & 0xf000) >> 12) as _;

            for pixel in pixel_buff.iter_mut() {
                *pixel = ((*pixel << 4) | *pixel)
            }
            buffer.put_slice(&pixel_buff);
            pixel_buff.fill(0)
        }
        Ok(())
    }
}
