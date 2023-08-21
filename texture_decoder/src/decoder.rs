use bytes::BufMut;
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

        if data_base_times != size_base_times {
            Err(DecodeImageError::SizeNotMatch(data_base_times, size_base_times))?;
        }
        Ok(())
    }

    fn decoding(size: &ImageSize, img_data: &[u8], buffer: &mut impl BufMut) -> Result<(), DecodeImageError> {
        Self::check_decodiblity(size, img_data.len())?;

        let image_chunks = img_data.chunks_exact(Self::DECODE_PIXEL_BYTE);
        for mut pixel_buf in image_chunks {
            Self::decode_pixel(&mut pixel_buf)?.write_buf(buffer);
        }
        Ok(())
    }

    fn decode_currently(size: &ImageSize, img_data: &[u8], buffer: &mut impl BufMut) -> Result<(), DecodeImageError> {
        Self::check_decodiblity(size, img_data.len())?;
        let image_chunks = img_data.chunks_exact(Self::DECODE_PIXEL_BYTE);
        let mut buf = WriteBuff::new(size.size() * Pixel::PIXEL_SPACE, Pixel::PIXEL_SPACE);
        image_chunks.zip(buf.to_chunks()).par_bridge().try_for_each(|(mut buff, mut write_buf)| {
            let pixels = Self::decode_pixel(&mut buff)?;
            pixels.write_buf(&mut write_buf);

            Ok::<_, io::Error>(())
        })?;
        buffer.put_slice(buf.as_slice());
        Ok(())
    }

    const DECODE_PIXEL_BYTE: usize;

    fn decode_pixel(data: &mut &[u8]) -> io::Result<[Pixel; PIXEL_NUM]>;
}
