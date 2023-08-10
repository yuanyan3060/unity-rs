pub mod error;
pub mod implements;
mod pixel_info;
mod utils;

pub struct Texture2DDecoder;

impl Texture2DDecoder {
    pub fn texture_decode_image<D: ImageDecoder>(size: &ImageSize, data: &[u8], flip: bool) -> Result<RgbaImage, DecodeImageError> {
        let mut buffer = Vec::new();
        buffer.reserve(size.output_size());
        D::decoding(size, data, &mut buffer)?;

        let img = <RgbaImage>::from_raw(size.width as _, size.height as _, buffer).ok_or(DecodeImageError::ImageDecode)?;

        if flip {
            Ok(flip_vertical(&img))
        } else {
            Ok(img)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageSize {
    width: usize,
    height: usize,
}

impl ImageSize {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn size(&self) -> usize {
        self.width * self.height
    }
    pub fn output_size(&self) -> usize {
        self.size() * 4
    }
}

use bytes::BufMut;
use image::imageops::flip_vertical;
use std::io;

use crate::error::DecodeImageError;
use crate::pixel_info::Pixel;
use image::RgbaImage;

pub trait ImageDecoder {
    fn decoding(size: &ImageSize, mut img_data: &[u8], buffer: &mut impl BufMut) -> io::Result<()> {
        let img = &mut img_data;
        for _ in 0..size.size() {
            Self::decode_step(img)?.write_but(buffer);
        }
        Ok(())
    }

    fn decode_step(data: &mut &[u8]) -> io::Result<Pixel>;
}
