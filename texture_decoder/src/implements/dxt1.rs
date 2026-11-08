use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use image::RgbaImage;

use crate::error::DecodeImageError;

pub struct DXT1;

impl DXT1 {
    pub fn decode(data: &[u8], width: u32, height: u32) -> Result<RgbaImage, DecodeImageError> {
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        let blocks_x = (width + 3) / 4;
        let block_size = 8;

        for (i, chunk) in data.chunks(block_size).enumerate() {
            if chunk.len() < block_size {
                break;
            }

            let pixels_in_block = Self::decode_block(chunk).map_err(|_| DecodeImageError::InvalidData)?;

            let block_x = (i as u32 % blocks_x) * 4;
            let block_y = (i as u32 / blocks_x) * 4;

            for row in 0..4 {
                for col in 0..4 {
                    let x = block_x + col;
                    let y = block_y + row;

                    if x >= width || y >= height {
                        continue;
                    }

                    let flipped_y = height - 1 - y;
                    let global_idx = ((flipped_y * width + x) * 4) as usize;
                    let pixel = pixels_in_block[(row * 4 + col) as usize];

                    buffer[global_idx..global_idx + 4].copy_from_slice(&pixel);
                }
            }
        }
        RgbaImage::from_raw(width, height, buffer).ok_or(DecodeImageError::ImageDecode)
    }

    fn decode_block(data: &[u8]) -> std::io::Result<[[u8; 4]; 16]> {
        let mut reader = Cursor::new(data);
        let c0 = reader.read_u16::<LittleEndian>()?;
        let c1 = reader.read_u16::<LittleEndian>()?;
        let color_idx = reader.read_u32::<LittleEndian>()?;

        let (r0, g0, b0) = Self::rgb565_to_rgb888(c0);
        let (r1, g1, b1) = Self::rgb565_to_rgb888(c1);

        let mut colors = [[0u8; 4]; 4];
        colors[0] = [r0, g0, b0, 255];
        colors[1] = [r1, g1, b1, 255];

        if c0 > c1 {
            colors[2] = [((2 * r0 as u16 + r1 as u16) / 3) as u8, ((2 * g0 as u16 + g1 as u16) / 3) as u8, ((2 * b0 as u16 + b1 as u16) / 3) as u8, 255];
            colors[3] = [((r0 as u16 + 2 * r1 as u16) / 3) as u8, ((g0 as u16 + 2 * g1 as u16) / 3) as u8, ((b0 as u16 + 2 * b1 as u16) / 3) as u8, 255];
        } else {
            colors[2] = [((r0 as u16 + r1 as u16) / 2) as u8, ((g0 as u16 + g1 as u16) / 2) as u8, ((b0 as u16 + b1 as u16) / 2) as u8, 255];
            colors[3] = [0, 0, 0, 0]; // 完全透明
        }

        let mut block_pixels = [[0u8; 4]; 16];
        for i in 0..16 {
            let ci = ((color_idx >> (2 * i)) & 0x3) as usize;
            block_pixels[i] = colors[ci];
        }
        Ok(block_pixels)
    }

    #[inline]
    fn rgb565_to_rgb888(c: u16) -> (u8, u8, u8) {
        let r = ((c >> 11) & 0x1f) as u8;
        let g = ((c >> 5) & 0x3f) as u8;
        let b = (c & 0x1f) as u8;
        ((r << 3) | (r >> 2), (g << 2) | (g >> 4), (b << 3) | (b >> 2))
    }
}
