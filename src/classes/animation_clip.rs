use crate::error::UnityResult;
use crate::math::Vector3;
use crate::object::ObjectInfo;
use crate::reader::Reader;

#[derive(Default, Debug)]
pub struct AnimationClip {
    pub center: Vector3,
    pub extent: Vector3,
}

impl AnimationClip {
    pub(super) fn load(r: &mut Reader) -> UnityResult<Self> {
        let center = r.read_vector3()?;
        let extent = r.read_vector3()?;
        Ok(Self { center, extent })
    }
}

#[derive(Debug)]
pub struct PackedFloatVector {
    pub num_items: u32,
    pub range: f32,
    pub start: f32,
    pub data: Vec<u8>,
    pub bit_size: u8,
}

impl PackedFloatVector {
    pub(super) fn load(_object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let ret = Self {
            num_items: r.read_u32()?,
            range: r.read_f32()?,
            start: r.read_f32()?,
            data: {
                let len = r.read_i32()? as usize;
                r.read_u8_list(len)?
            },
            bit_size: r.read_u8()?,
        };
        r.align(4)?;
        Ok(ret)
    }

    pub fn unpack_floats(&self, item_count_in_chunk: usize, chunk_stride: usize, start: usize, num_chunks: Option<usize>) -> Vec<f32> {
        let bit_pos = self.bit_size as usize * start;
        let mut index_pos = bit_pos / 8;
        let mut bit_pos = bit_pos % 8;

        let scale = 1.0 / self.range;
        let num_chunks = num_chunks.unwrap_or((self.num_items as usize + item_count_in_chunk - 1) / item_count_in_chunk);
        let end = chunk_stride * num_chunks / 4;
        let mut data = Vec::with_capacity(end);

        let mut index = 0;
        while index < end {
            for _ in 0..item_count_in_chunk {
                let mut x: u32 = 0;
                let mut bits = 0;

                while bits < self.bit_size as usize {
                    let byte = self.data[index_pos];
                    let num = std::cmp::min(self.bit_size as usize - bits, 8 - bit_pos);
                    x |= ((byte >> bit_pos) as u32) << bits;
                    bit_pos += num;
                    bits += num;
                    if bit_pos == 8 {
                        index_pos += 1;
                        bit_pos = 0;
                    }
                }
                x &= (1 << self.bit_size) - 1;
                let float_value = (x as f32) / (scale * ((1 << self.bit_size) as f32 - 1.0)) + self.start;
                data.push(float_value);
            }
            index += chunk_stride / 4;
        }
        data
    }
}

#[derive(Debug)]
pub struct PackedIntVector {
    pub num_items: u32,
    pub data: Vec<u8>,
    pub bit_size: u8,
}

impl PackedIntVector {
    pub(super) fn load(_object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let ret = Self {
            num_items: r.read_u32()?,
            data: {
                let len = r.read_i32()? as usize;
                r.read_u8_list(len)?
            },
            bit_size: {
                r.align(4)?;
                r.read_u8()?
            },
        };
        r.align(4)?;
        Ok(ret)
    }

    pub fn unpack_ints(&self) -> Vec<i32> {
        let mut unpacked_data = vec![0; self.num_items as usize];
        let mut index_pos = 0;
        let mut bit_pos = 0;

        for i in 0..self.num_items {
            let mut bits = 0;
            unpacked_data[i as usize] = 0;

            while bits < self.bit_size as usize {
                unpacked_data[i as usize] |= ((self.data[index_pos] >> bit_pos) as i32) << bits;
                let num = std::cmp::min(self.bit_size as usize - bits, 8 - bit_pos);
                bit_pos += num;
                bits += num;

                if bit_pos == 8 {
                    index_pos += 1;
                    bit_pos = 0;
                }
            }
            unpacked_data[i as usize] &= (1 << self.bit_size) - 1;
        }
        unpacked_data
    }
}
