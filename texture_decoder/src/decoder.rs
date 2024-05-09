use std::io;

use crate::error::DecodeImageError;
use crate::pixel_info::{Pixel, WritePixelBuf};
use crate::write_buffer::WriteBuff;
use crate::ImageSize;
use rayon::iter::{ParallelBridge, ParallelIterator};

pub trait ImageDecoder<const PIXEL_NUM: usize = 1> {
    fn check_decodiblity(size: &ImageSize, data_len: usize) -> Result<(), DecodeImageError> {
        let data_base_times = data_len / Self::DECODE_PIXEL_BYTE;
        let size_base_times = size.size() / PIXEL_NUM;

        if data_base_times < size_base_times {
            Err(DecodeImageError::SizeNotMatch(data_base_times, size_base_times))?;
        }
        Ok(())
    }

    fn decoding(size: &ImageSize, img_data: &[u8]) -> Result<Box<[u8]>, DecodeImageError> {
        Self::check_decodiblity(size, img_data.len())?;

        let image_chunks = img_data.chunks_exact(Self::DECODE_PIXEL_BYTE);
        let mut out_buff = WriteBuff::new(size.output_size(), Pixel::PIXEL_SPACE * PIXEL_NUM);
        for (mut pixel_buf, mut out_buf) in image_chunks.zip(out_buff.as_chunks()) {
            Self::decode_pixel(&mut pixel_buf)?.write_buf(&mut out_buf);
        }
        Ok(out_buff.inner())
    }

    fn decode_currently(size: &ImageSize, img_data: &[u8]) -> Result<Box<[u8]>, DecodeImageError> {
        Self::check_decodiblity(size, img_data.len())?;
        let image_chunks = img_data.chunks_exact(Self::DECODE_PIXEL_BYTE);
        let mut buf = WriteBuff::new(size.output_size(), Pixel::PIXEL_SPACE * PIXEL_NUM);
        image_chunks.zip(buf.as_chunks()).par_bridge().try_for_each(|(mut buff, mut write_buf)| {
            let pixels = Self::decode_pixel(&mut buff)?;
            pixels.write_buf(&mut write_buf);

            Ok::<_, io::Error>(())
        })?;
        Ok(buf.inner())
    }

    const DECODE_PIXEL_BYTE: usize;

    fn decode_pixel(data: &mut &[u8]) -> io::Result<[Pixel; PIXEL_NUM]>;
}
