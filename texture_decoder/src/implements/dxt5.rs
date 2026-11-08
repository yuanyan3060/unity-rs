use byteorder::{LittleEndian, ReadBytesExt};
use image::RgbaImage;
use std::io::{self, Cursor, Read};

pub struct DXT5;

impl DXT5 {
    pub fn decode(data: &[u8], width: u32, height: u32) -> io::Result<RgbaImage> {
        let mut buffer = vec![0u8; (width * height * 4) as usize];

        let blocks_x = (width + 3) / 4;
        let block_size = 16;

        for (i, chunk) in data.chunks(block_size).enumerate() {
            if chunk.len() < block_size {
                break;
            }

            let pixels_in_block = Self::decode_block(chunk)?;

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

                    let local_idx = (row * 4 + col) as usize;
                    let pixel = pixels_in_block[local_idx];

                    buffer[global_idx] = pixel[0];
                    buffer[global_idx + 1] = pixel[1];
                    buffer[global_idx + 2] = pixel[2];
                    buffer[global_idx + 3] = pixel[3];
                }
            }
        }
        Ok(RgbaImage::from_raw(width, height, buffer).unwrap())
    }

    fn decode_block(data: &[u8]) -> std::io::Result<[[u8; 4]; 16]> {
        let mut reader = Cursor::new(data);

        let alpha0 = reader.read_u8()?;
        let alpha1 = reader.read_u8()?;

        let mut alpha_indices_buf = [0u8; 6];
        reader.read_exact(&mut alpha_indices_buf)?;
        let mut alpha_idx_u64 = 0u64;
        for (i, &b) in alpha_indices_buf.iter().enumerate() {
            alpha_idx_u64 |= (b as u64) << (8 * i);
        }
        let mut alphas = [0u8; 8];
        alphas[0] = alpha0;
        alphas[1] = alpha1;
        if alpha0 > alpha1 {
            for i in 2..8 {
                alphas[i] = (((8 - i) as u16 * alpha0 as u16 + (i - 1) as u16 * alpha1 as u16) / 7) as u8;
            }
        } else {
            for i in 2..6 {
                alphas[i] = (((6 - i) as u16 * alpha0 as u16 + (i - 1) as u16 * alpha1 as u16) / 5) as u8;
            }
            alphas[6] = 0;
            alphas[7] = 255;
        }

        let c0 = reader.read_u16::<LittleEndian>()?;
        let c1 = reader.read_u16::<LittleEndian>()?;
        let color_idx = reader.read_u32::<LittleEndian>()?;

        let (r0, g0, b0) = Self::rgb565_to_rgb888(c0);
        let (r1, g1, b1) = Self::rgb565_to_rgb888(c1);

        let mut colors = [[0u8; 3]; 4];
        colors[0] = [r0, g0, b0];
        colors[1] = [r1, g1, b1];
        colors[2] = [((2 * r0 as u16 + r1 as u16) / 3) as u8, ((2 * g0 as u16 + g1 as u16) / 3) as u8, ((2 * b0 as u16 + b1 as u16) / 3) as u8];
        colors[3] = [((r0 as u16 + 2 * r1 as u16) / 3) as u8, ((g0 as u16 + 2 * g1 as u16) / 3) as u8, ((b0 as u16 + 2 * b1 as u16) / 3) as u8];

        let mut block_pixels = [[0u8; 4]; 16];
        for i in 0..16 {
            let ai = ((alpha_idx_u64 >> (3 * i)) & 0x7) as usize;
            let ci = ((color_idx >> (2 * i)) & 0x3) as usize;

            let rgb = colors[ci];
            let a = alphas[ai];

            block_pixels[i] = [rgb[0], rgb[1], rgb[2], a];
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
