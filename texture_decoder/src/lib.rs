pub mod error;
pub mod implments;
mod utils;

pub struct Texture2DDecoder;

impl Texture2DDecoder {
    pub fn texture_decode_image<D: ImageDecoder>(size: &ImageSize, data: &[u8]) -> Result<RgbaImage, DecodeImageError> {
        let mut buffer = Vec::new();
        buffer.reserve(size.output_size());
        D::decoding(size, data, &mut buffer)?;
        let mut chunks = buffer.chunks_exact_mut(4);
        // bgra -> rgba
        while let Some([b, _, r, _]) = chunks.next() {
            let &mut t = r;
            *r = *b;
            *b = t;
        }

        let img = <RgbaImage>::from_raw(size.width as _, size.height as _, buffer).ok_or(DecodeImageError::ImageDecode)?;

        Ok(img)
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
use std::io;

use crate::error::DecodeImageError;
use image::RgbaImage;

pub trait ImageDecoder {
    fn decoding(size: &ImageSize, img_data: &[u8], buffer: &mut impl BufMut) -> io::Result<()>;
}
