use crate::error::{UnityError, UnityResult};
use crate::math::{Matrix4x4, RectF32, Vector2, Vector3, Vector4};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    Big,
    Little,
}

#[derive(Clone)]
pub struct Reader<'a> {
    buf: &'a [u8],
    offset: usize,
    order: ByteOrder,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8], order: ByteOrder) -> Self {
        Self { buf, offset: 0, order }
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn set_offset(&mut self, offset: usize) -> UnityResult<usize> {
        if self.buf.len() < offset {
            return Err(UnityError::Eof);
        }
        let result = self.offset;
        self.offset = offset;
        Ok(result)
    }

    pub fn set_little_order(&mut self) {
        self.order = ByteOrder::Little
    }

    pub fn set_big_order(&mut self) {
        self.order = ByteOrder::Big
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_order(&self) -> ByteOrder {
        self.order
    }

    pub fn has_space(&self, length: usize) -> UnityResult<usize> {
        let end = length + self.get_offset();
        if self.buf.len() < end {
            Err(UnityError::Eof)
        } else {
            Ok(end)
        }
    }

    pub fn align(&mut self, num: usize) -> UnityResult<usize> {
        let offset_before_align = self.get_offset();
        let remain = offset_before_align % num;
        let offset_after_align = match remain {
            0 => offset_before_align,
            _ => offset_before_align - remain + num,
        };
        if offset_after_align > self.buf.len() {
            return Err(UnityError::Eof);
        }
        self.set_offset(offset_after_align)?;
        Ok(offset_after_align)
    }

    pub fn read_u8_slice(&mut self, length: usize) -> UnityResult<&[u8]> {
        let end = self.has_space(length)?;
        let result = &self.buf[self.offset..end];
        self.offset = end;
        Ok(result)
    }

    pub fn read_u8_array<const N: usize>(&mut self) -> UnityResult<[u8; N]> {
        let mut result = [0; N];
        let end = self.has_space(N)?;
        let slice = &self.buf[self.get_offset()..end];
        result.copy_from_slice(slice);
        self.offset = end;
        Ok(result)
    }

    pub fn read_f32_array<const N: usize>(&mut self) -> UnityResult<[f32; N]> {
        let mut result = [0.0; N];
        let end = self.has_space(N * std::mem::size_of::<f32>())?;
        for item in result.iter_mut() {
            *item = self.read_f32()?;
        }
        self.offset = end;
        Ok(result)
    }

    pub fn read_i32_array<const N: usize>(&mut self) -> UnityResult<[i32; N]> {
        let mut result = [0; N];
        let end = self.has_space(N * std::mem::size_of::<i32>())?;
        for item in result.iter_mut() {
            *item = self.read_i32()?;
        }
        self.offset = end;
        Ok(result)
    }

    pub fn read_u8_list(&mut self, length: usize) -> UnityResult<Vec<u8>> {
        Ok(self.read_u8_slice(length)?.to_vec())
    }

    pub fn read_u8(&mut self) -> UnityResult<u8> {
        let end = self.has_space(1)?;
        let result = self.buf[self.offset];
        self.offset = end;
        Ok(result)
    }

    pub fn read_bool(&mut self) -> UnityResult<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_u16(&mut self) -> UnityResult<u16> {
        let a = self.read_u8_array::<2>()?;
        match self.order {
            ByteOrder::Big => Ok(u16::from_be_bytes(a)),
            ByteOrder::Little => Ok(u16::from_le_bytes(a)),
        }
    }

    pub fn read_u32(&mut self) -> UnityResult<u32> {
        let a = self.read_u8_array::<4>()?;
        match self.order {
            ByteOrder::Big => Ok(u32::from_be_bytes(a)),
            ByteOrder::Little => Ok(u32::from_le_bytes(a)),
        }
    }

    pub fn read_u64(&mut self) -> UnityResult<u64> {
        let a = self.read_u8_array::<8>()?;
        match self.order {
            ByteOrder::Big => Ok(u64::from_be_bytes(a)),
            ByteOrder::Little => Ok(u64::from_le_bytes(a)),
        }
    }

    pub fn read_i8(&mut self) -> UnityResult<i8> {
        let a = self.read_u8_array::<1>()?;
        Ok(i8::from_be_bytes(a))
    }

    pub fn read_i16(&mut self) -> UnityResult<i16> {
        let a = self.read_u8_array::<2>()?;
        match self.order {
            ByteOrder::Big => Ok(i16::from_be_bytes(a)),
            ByteOrder::Little => Ok(i16::from_le_bytes(a)),
        }
    }

    pub fn read_i32(&mut self) -> UnityResult<i32> {
        let a = self.read_u8_array::<4>()?;
        match self.order {
            ByteOrder::Big => Ok(i32::from_be_bytes(a)),
            ByteOrder::Little => Ok(i32::from_le_bytes(a)),
        }
    }

    pub fn read_i64(&mut self) -> UnityResult<i64> {
        let a = self.read_u8_array::<8>()?;
        match self.order {
            ByteOrder::Big => Ok(i64::from_be_bytes(a)),
            ByteOrder::Little => Ok(i64::from_le_bytes(a)),
        }
    }

    pub fn read_f32(&mut self) -> UnityResult<f32> {
        let a = self.read_u8_array::<4>()?;
        match self.order {
            ByteOrder::Big => Ok(f32::from_be_bytes(a)),
            ByteOrder::Little => Ok(f32::from_le_bytes(a)),
        }
    }

    pub fn read_f64(&mut self) -> UnityResult<f64> {
        let a = self.read_u8_array::<8>()?;
        match self.order {
            ByteOrder::Big => Ok(f64::from_be_bytes(a)),
            ByteOrder::Little => Ok(f64::from_le_bytes(a)),
        }
    }

    pub fn read_7bit_u32(&mut self) -> UnityResult<u32> {
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

    pub fn read_string_util_null(&mut self) -> UnityResult<String> {
        let mut ret = Vec::new();
        loop {
            let b = self.read_u8()?;
            if b == 0 {
                break;
            } else {
                ret.push(b);
            }
        }
        Ok(String::from_utf8_lossy(&ret).to_string())
    }

    pub fn read_string_util_null_with_limit(&mut self, limit: usize) -> UnityResult<String> {
        let mut ret = Vec::new();
        for _ in 0..limit {
            let b = self.read_u8()?;
            if b == 0 {
                break;
            } else {
                ret.push(b);
            }
        }
        Ok(String::from_utf8_lossy(&ret).to_string())
    }

    pub fn read_u8_list_util_null_with_limit(&mut self, limit: usize) -> UnityResult<Vec<u8>> {
        let mut ret = Vec::new();
        for _ in 0..limit {
            let b = self.read_u8()?;
            if b == 0 {
                break;
            } else {
                ret.push(b);
            }
        }
        Ok(ret)
    }

    pub fn read_string_with_length(&mut self, length: usize) -> UnityResult<String> {
        Ok(String::from_utf8_lossy(self.read_u8_slice(length)?).to_string())
    }

    pub fn read_string_with_7bit_length(&mut self) -> UnityResult<String> {
        let length = self.read_7bit_u32()?;
        self.read_string_with_length(length as usize)
    }

    pub fn read_aligned_string(&mut self) -> UnityResult<String> {
        let length = self.read_i32()?;
        let result = self.read_string_with_length(length as usize);
        self.align(4)?;
        result
    }

    pub fn read_i32_list(&mut self, length: usize) -> UnityResult<Vec<i32>> {
        self.has_space(length)?;
        let mut ret = Vec::with_capacity(length);
        for _ in 0..length {
            ret.push(self.read_i32()?)
        }
        Ok(ret)
    }

    pub fn read_rect_f32(&mut self) -> UnityResult<RectF32> {
        Ok(RectF32 {
            x: self.read_f32()?,
            y: self.read_f32()?,
            w: self.read_f32()?,
            h: self.read_f32()?,
        })
    }

    pub fn read_vector2(&mut self) -> UnityResult<Vector2> {
        Ok(Vector2 { x: self.read_f32()?, y: self.read_f32()? })
    }

    pub fn read_vector3(&mut self) -> UnityResult<Vector3> {
        Ok(Vector3 {
            x: self.read_f32()?,
            y: self.read_f32()?,
            z: self.read_f32()?,
        })
    }

    pub fn read_vector4(&mut self) -> UnityResult<Vector4> {
        Ok(Vector4 {
            x: self.read_f32()?,
            y: self.read_f32()?,
            z: self.read_f32()?,
            w: self.read_f32()?,
        })
    }

    pub fn read_matrix4x4(&mut self) -> UnityResult<Matrix4x4> {
        Ok(Matrix4x4 {
            m00: self.read_f32()?,
            m10: self.read_f32()?,
            m20: self.read_f32()?,
            m30: self.read_f32()?,
            m01: self.read_f32()?,
            m11: self.read_f32()?,
            m21: self.read_f32()?,
            m31: self.read_f32()?,
            m02: self.read_f32()?,
            m12: self.read_f32()?,
            m22: self.read_f32()?,
            m32: self.read_f32()?,
            m03: self.read_f32()?,
            m13: self.read_f32()?,
            m23: self.read_f32()?,
            m33: self.read_f32()?,
        })
    }

    pub fn read_u16_list(&mut self, size: usize) -> UnityResult<Vec<u16>> {
        let _end = self.has_space(size)?;
        let mut ret = Vec::with_capacity(size);
        for _ in 0..size {
            ret.push(self.read_u16()?)
        }
        Ok(ret)
    }

    pub fn read_u32_list(&mut self, size: usize) -> UnityResult<Vec<u32>> {
        let _end = self.has_space(size)?;
        let mut ret = Vec::with_capacity(size);
        for _ in 0..size {
            ret.push(self.read_u32()?)
        }
        Ok(ret)
    }

    pub fn read_string_list(&mut self) -> UnityResult<Vec<String>> {
        let length = self.read_i32()?;
        let mut result = Vec::with_capacity(length as usize);
        for _ in 0..length {
            result.push(self.read_aligned_string()?);
        }
        Ok(result)
    }

    pub fn read_f32_list(&mut self, size: usize) -> UnityResult<Vec<f32>> {
        let _end = self.has_space(size)?;
        let mut ret = Vec::with_capacity(size);
        for _ in 0..size {
            ret.push(self.read_f32()?)
        }
        Ok(ret)
    }

    pub fn read_matrix4x4_list(&mut self, size: usize) -> UnityResult<Vec<Matrix4x4>> {
        let _end = self.has_space(size)?;
        let mut ret = Vec::with_capacity(size);
        for _ in 0..size {
            ret.push(self.read_matrix4x4()?)
        }
        Ok(ret)
    }
}
