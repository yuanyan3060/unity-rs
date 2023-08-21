use bytes::BufMut;
use typed_builder::TypedBuilder;

pub type SinglePixel = [Pixel; 1];

#[derive(Debug, Copy, Clone, Default, TypedBuilder)]
pub struct Pixel {
    #[builder(default = 0)]
    rad: u8,
    #[builder(default = 0)]
    green: u8,
    #[builder(default = 0)]
    blue: u8,
    #[builder(default = 255)]
    alpha: u8,
}

impl Into<SinglePixel> for Pixel {
    fn into(self) -> SinglePixel {
        [self]
    }
}

impl Pixel {
    pub const PIXEL_SPACE: usize = 4;

    pub(crate) fn new_rgba(rad: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self { rad, green, blue, alpha }
    }
    pub(crate) fn new_rgb(rad: u8, green: u8, blue: u8) -> Self {
        Self::new_rgba(rad, green, blue, 255)
    }

    pub(crate) fn as_array(&self) -> [u8; 4] {
        [self.rad, self.green, self.blue, self.alpha]
    }
}

pub trait WritePixelBuf {
    fn write_buf(&self, buffer: &mut impl BufMut);
}

impl WritePixelBuf for Pixel {
    fn write_buf(&self, buffer: &mut impl BufMut) {
        buffer.put_slice(&self.to_slice())
    }
}

impl<const N: usize> WritePixelBuf for [Pixel; N] {
    fn write_buf(&self, buffer: &mut impl BufMut) {
        self.into_iter().for_each(|pixel| pixel.write_buf(buffer))
    }
}
