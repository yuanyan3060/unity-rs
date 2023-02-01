use bytes::Bytes;
use super::{error::Error, math::{RectF32, Vector2, Vector3, Vector4}};
#[derive(Clone, Copy)]
pub enum ByteOrder {
    Big,
    Little,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Reader{
    data: bytes::Bytes,
    order: ByteOrder,
    offset: usize,
}

impl Reader {
    pub fn new(src: bytes::Bytes, order: ByteOrder) -> Self {
        Reader {
            offset: 0,
            data: src,
            order: order,
        }
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn set_offset(&mut self, offset: usize) -> Result<()> {
        if self.data.len() < offset {
            return Err(Error::EOF);
        }
        self.offset = offset;
        Ok(())
    }

    pub fn set_little_order(&mut self) {
        self.order = ByteOrder::Little
    }

    pub fn set_big_order(&mut self) {
        self.order = ByteOrder::Big
    }

    pub fn get_size(&self) -> usize {
        self.data.len()
    }

    pub fn has_space(&self, target: usize) -> Result<usize> {
        let end = target + self.get_offset();
        if self.data.len() < end {
            Err(Error::EOF)
        } else {
            Ok(end)
        }
    }
    fn read_list<T, F:Fn(&mut Reader)->Result<T>>(&mut self, func:F, length: usize)-> Result<Vec<T>>{
        let mut result = Vec::with_capacity(length);
        for i in 0..length{
            let v = func(self)?;
            result.push(v);
        }
        Ok(result)
    }

    pub fn read_u8_list(&mut self, size: usize) -> Result<Bytes> {
        let end = self.has_space(size)?;
        let ret = self.data.slice(self.offset..end);
        self.set_offset(end)?;
        Ok(ret)
    }

    pub fn read_u8_slice(&mut self, size: usize) -> Result<Bytes> {
        self.read_u8_list(size)
    }

    pub fn read_u8_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut a = [0; N];
        let end = self.get_offset() + N;
        if self.data.len() < end {
            return Err(Error::EOF);
        }
        let slice = &self.data[self.get_offset()..end];
        a.copy_from_slice(slice);
        self.set_offset(end)?;
        Ok(a)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let end = self.has_space(1)?;
        let result = self.data[self.get_offset()];
        self.set_offset(end)?;
        Ok(result)
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let a = self.read_u8_array::<2>()?;
        match self.order{
            ByteOrder::Big => Ok(u16::from_be_bytes(a)),
            ByteOrder::Little => Ok(u16::from_le_bytes(a)),
        }
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let a = self.read_u8_array::<4>()?;
        match self.order {
            ByteOrder::Big => Ok(u32::from_be_bytes(a)),
            ByteOrder::Little => Ok(u32::from_le_bytes(a)),
        }
    }

    pub fn read_u64(&mut self) -> Result<u64> {
        let a = self.read_u8_array::<8>()?;
        match self.order {
            ByteOrder::Big => Ok(u64::from_be_bytes(a)),
            ByteOrder::Little => Ok(u64::from_le_bytes(a)),
        }
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        let a = self.read_u8_array::<1>()?;
        Ok(i8::from_be_bytes(a))
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        let a = self.read_u8_array::<2>()?;
        match self.order {
            ByteOrder::Big => Ok(i16::from_be_bytes(a)),
            ByteOrder::Little => Ok(i16::from_le_bytes(a)),
        }
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let a = self.read_u8_array::<4>()?;
        match self.order {
            ByteOrder::Big => Ok(i32::from_be_bytes(a)),
            ByteOrder::Little => Ok(i32::from_le_bytes(a)),
        }
    }

    pub fn read_i32_list(&mut self, size: usize) -> Result<Vec<i32>> {
        let end = self.has_space(size)?;
        let mut ret = Vec::with_capacity(size);
        for _ in 0..size {
            ret.push(self.read_i32()?)
        }
        Ok(ret)
    }

    pub fn read_i32_list_without_size(&mut self) -> Result<Vec<i32>> {
        let size = self.read_i32()? as usize;
        let end = self.has_space(size)?;
        let mut ret = Vec::with_capacity(size);
        for _ in 0..size {
            ret.push(self.read_i32()?)
        }
        Ok(ret)
    }

    pub fn read_i64(&mut self) -> Result<i64> {
        let a = self.read_u8_array::<8>()?;
        match self.order {
            ByteOrder::Big => Ok(i64::from_be_bytes(a)),
            ByteOrder::Little => Ok(i64::from_le_bytes(a)),
        }
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        let a = self.read_u8_array::<4>()?;
        match self.order {
            ByteOrder::Big => Ok(f32::from_be_bytes(a)),
            ByteOrder::Little => Ok(f32::from_le_bytes(a)),
        }
    }

    pub fn read_f64(&mut self) -> Result<f64> {
        let a = self.read_u8_array::<8>()?;
        match self.order {
            ByteOrder::Big => Ok(f64::from_be_bytes(a)),
            ByteOrder::Little => Ok(f64::from_le_bytes(a)),
        }
    }

    pub fn read_7bit_u32(&mut self) -> Result<u32> {
        let mut out = 0u32;
        let mut shift = 0u32;
        loop {
            let b = self.read_u8()?;
            out |= ((b & 0x7f) as u32) << shift;
            shift += 7;
            if b & 0x80 == 0 {
                break;
            }
        }
        Ok(out)
    }

    pub fn read_string_utill_null(&mut self) -> Result<String> {
        let mut out = String::new();
        loop {
            let b = self.read_u8()?;
            if b == 0 {
                break;
            } else {
                out.push(b as char);
            }
        }
        Ok(out)
    }

    pub fn read_string_utill_null_limit(&mut self, size:usize) -> Result<String> {
        let mut out = String::new();
        for _ in 0..size{
            let b = self.read_u8()?;
            if b == 0 {
                break;
            } else {
                out.push(b as char);
            }
        }
        Ok(out)
    }

    pub fn read_string_with_length(&mut self, length: usize) -> Result<String> {
        let mut out = String::with_capacity(length);
        for _ in 0..length {
            let b = self.read_u8()?;
            out.push(b as char);
        }
        Ok(out)
    }

    pub fn read_string(&mut self) -> Result<String> {
        let length = self.read_7bit_u32()?;
        self.read_string_with_length(length as usize)
    }

    pub fn read_aligned_string(&mut self) -> Result<String> {
        let length = self.read_i32()?;
        let result = self.read_string_with_length(length as usize);
        self.align(4)?;
        result

    }
    

    pub fn align(&mut self, num: usize) -> Result<usize> {
        let offset_before_align = self.get_offset();
        let remain = offset_before_align % num;
        let offset_after_align = match remain {
            0 => offset_before_align,
            _ => offset_before_align - remain + num,
        };
        if offset_after_align > self.data.len() {
            return Err(Error::EOF);
        }
        self.set_offset(offset_after_align)?;
        Ok(offset_after_align)
    }

    pub fn read_rect_f32(&mut self) -> Result<RectF32> {
        Ok(RectF32{
            x: self.read_f32()?,
            y: self.read_f32()?,
            w: self.read_f32()?,
            h: self.read_f32()?,
        })
    }

    pub fn read_vector2(&mut self) -> Result<Vector2> {
        Ok(Vector2{
            x: self.read_f32()?,
            y: self.read_f32()?,
        })
    }

    pub fn read_vector3(&mut self) -> Result<Vector3> {
        Ok(Vector3{
            x: self.read_f32()?,
            y: self.read_f32()?,
            z: self.read_f32()?,
        })
    }

    pub fn read_vector4(&mut self) -> Result<Vector4> {
        Ok(Vector4{
            x: self.read_f32()?,
            y: self.read_f32()?,
            z: self.read_f32()?,
            w: self.read_f32()?,
        })
    }

    pub fn read_string_list(&mut self)->Result<Vec<String>>{
        let length = self.read_i32()?;
        Ok(self.read_list(Self::read_aligned_string, length as usize)?)
    }

    pub fn read_u16_list(&mut self, size: usize) -> Result<Vec<u16>> {
        let end = self.has_space(size)?;
        let mut ret = Vec::with_capacity(size);
        for _ in 0..size {
            ret.push(self.read_u16()?)
        }
        Ok(ret)
    }

}
