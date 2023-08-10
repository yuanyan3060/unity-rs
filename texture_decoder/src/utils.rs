use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use half::f16;
use std::io;
use std::num::Wrapping;

pub(crate) trait DownScaleToU8 {
    fn down_scale(self) -> u8;
}

impl DownScaleToU8 for u16 {
    fn down_scale(self) -> u8 {
        let warp = Wrapping(self);
        (((warp * Wrapping(255)) + Wrapping(32895)) >> 16).0 as _
    }
}

pub(crate) fn clamp_byte(x: u16) -> u8 {
    if let Ok(v) = x.try_into() {
        v
    } else {
        u8::MAX
    }
}

pub(crate) trait ByteOrderExt {
    fn read_f16(buf: [u8; 2]) -> f16;
}

impl ByteOrderExt for BigEndian {
    fn read_f16(buf: [u8; 2]) -> f16 {
        f16::from_be_bytes(buf)
    }
}
impl ByteOrderExt for LittleEndian {
    fn read_f16(buf: [u8; 2]) -> f16 {
        f16::from_le_bytes(buf)
    }
}

pub(crate) trait ReadHalfFloat: ReadBytesExt {
    fn read_f16<T: ByteOrderExt>(&mut self) -> io::Result<f16> {
        let buf = [self.read_u8()?, self.read_u8()?];
        Ok(T::read_f16(buf))
    }
}

impl<T> ReadHalfFloat for T where T: ReadBytesExt {}

pub(crate) trait FloatConvU8 {
    fn to_u8(self) -> u8;
}

impl FloatConvU8 for f16 {
    fn to_u8(self) -> u8 {
        let full = self.to_f32();
        (full * 255f32).round() as _
    }
}

impl FloatConvU8 for f32 {
    fn to_u8(self) -> u8 {
        (self * 255f32).round() as _
    }
}
impl FloatConvU8 for f64 {
    fn to_u8(self) -> u8 {
        (self * 255f64).round() as _
    }
}
