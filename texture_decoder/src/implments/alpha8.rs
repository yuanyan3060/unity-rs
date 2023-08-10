use std::io;
use bytes::{BufMut, BytesMut};
use crate::{ImageDecoder, ImageSize};

pub struct Alpha8;

impl ImageDecoder for Alpha8 {
    fn decoding(size: &ImageSize, img_data: &[u8], buffer: &mut impl BufMut)->io::Result<()> {
        let size = size.size();
        for (_idx,&data) in (0..size).zip(img_data){
            buffer.put_slice(&[0xFF,0xFF,0xFF,data]);
        }
        Ok(())
    }
}
