mod decoder;
pub mod error;
pub mod implements;
mod pixel_info;
mod utils;
mod write_buffer;

pub struct Texture2DDecoder;

impl Texture2DDecoder {
    pub fn decode<const N: usize, D: ImageDecoder<N>>(_: D, size: &ImageSize, data: &[u8], flip: bool) -> Result<RgbaImage, DecodeImageError> {
       let buffer =  D::decode_currently(size, data)?.to_vec();

        let img = RgbaImage::from_raw(size.width as _, size.height as _, buffer).ok_or(DecodeImageError::ImageDecode)?;

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
        self.size() * Pixel::PIXEL_SPACE
    }
}

use crate::error::DecodeImageError;
pub use decoder::ImageDecoder;
use image::imageops::flip_vertical;
use image::RgbaImage;
use crate::pixel_info::Pixel;
