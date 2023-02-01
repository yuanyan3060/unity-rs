#![allow(non_snake_case)]
use bytes::Bytes;

use crate::unity::{object::ObjectInfo, Error, FromObject, Object, Reader, Result};

use super::animation_clip::AABB;
use num_enum::TryFromPrimitive;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(i32)]
pub enum GfxPrimitiveType {
    Triangles = 0,
    TriangleStrip = 1,
    Quads = 2,
    Lines = 3,
    LineStrip = 4,
    Points = 5,
}

impl std::default::Default for GfxPrimitiveType {
    fn default() -> Self {
        Self::Triangles
    }
}

#[derive(Default, Debug)]
pub struct SubMesh {
    pub first_bytes: u8,
    pub index_count: u8,
    pub topology: GfxPrimitiveType,
    pub triangle_count: u8,
    pub base_vertex: u8,
    pub first_vertex: u8,
    pub vertex_count: u8,
    pub local_aabb: Option<AABB>,
}

impl SubMesh {
    pub(super) fn load(object: &Object, r: &mut Reader) -> Result<Self> {
        let mut result = Self::default();
        let version = object.info.version;
        result.first_bytes = r.read_u32()? as u8;
        result.index_count = r.read_u32()? as u8;
        result.topology = r.read_i32()?.try_into().or(Err(Error::InvalidValue))?;
        if (version[0] < 4) {
            result.triangle_count = r.read_u32()? as u8;
        }

        if (version[0] > 2017 || (version[0] == 2017 && version[1] >= 3)) {
            result.base_vertex = r.read_u32()? as u8;
        }

        if (version[0] >= 3) {
            result.first_vertex = r.read_u32()? as u8;
            result.vertex_count = r.read_u32()? as u8;
            result.local_aabb = Some(AABB::load(&object, r)?);
        }
        Ok(result)
    }
}

#[derive(Default, Debug)]
pub struct ChannelInfo {
    pub stream: u8,
    pub offset: u8,
    pub format: u8,
    pub dimension: u8,
}

impl ChannelInfo {
    pub(super) fn load(object: &Object, r: &mut Reader) -> Result<Self> {
        let mut result = Self::default();
        result.stream = r.read_u8()?;
        result.offset = r.read_u8()?;
        result.format = r.read_u8()?;
        result.dimension = r.read_u8()? & 0xF;
        Ok(result)
    }
}

#[derive(Default, Debug)]
pub struct StreamInfo {
    pub channelMask: u8,
    pub offset: u8,
    pub stride: u8,
    pub align: u8,
    pub dividerOp: u8,
    pub frequency: u16,
}

impl StreamInfo {
    pub(super) fn load(object: &Object, r: &mut Reader) -> Result<Self> {
        let version = object.info.version;
        let mut result = Self::default();
        result.channelMask = r.read_u8()?;
        result.offset = r.read_u8()?;
        if version[0] < 4 {
            result.stride = r.read_u32()? as u8;
            result.align = r.read_u32()? as u8;
        } else {
            result.stride = r.read_u8()?;
            result.dividerOp = r.read_u8()?;
            result.frequency = r.read_u16()?;
        }
        Ok(result)
    }
}
#[derive(Default, Debug)]
pub struct VertexData {
    pub m_CurrentChannels: u8,
    pub m_VertexCount: u8,
    pub m_Channels: Vec<ChannelInfo>,
    pub m_Streams: Vec<StreamInfo>,
    pub m_DataSize: Bytes,
}

impl VertexData {
    pub(super) fn load(object: &Object, r: &mut Reader) -> Result<Self> {
        let version = object.info.version;
        let mut result = Self::default();
        if (version[0] < 2018) {
            result.m_CurrentChannels = r.read_u32()? as u8;
        }

        result.m_VertexCount = r.read_u32()? as u8;

        if (version[0] >= 4) {
            let size = r.read_i32()?;
            for _ in 0..size {
                result.m_Channels.push(ChannelInfo::load(object, r)?)
            }
        }
        if version[0] < 5 {
            if version[0] < 4 {
                result.m_Streams = Vec::with_capacity(4);
            } else {
                result.m_Streams = Vec::with_capacity(r.read_i32()? as usize);
            }
            for i in 0..result.m_Streams.capacity() {
                result.m_Streams.push(StreamInfo::load(object, r)?)
            }
            if version[0] < 4 {
                result.get_channels(version)?;
            }
        } else {
            result.get_streams(version)?;
        }
        let size = r.read_i32()?;
        result.m_DataSize = r.read_u8_list(size as usize)?;
        Ok(result)
    }
    fn get_channels(&mut self, version: [i32; 4]) -> Result<()> {
        self.m_Channels = Vec::with_capacity(6);
        for _ in 0..6 {
            self.m_Channels.push(ChannelInfo::default())
        }
        for s in 0..self.m_Streams.len() {
            let mut channelMask = self.m_Streams[s].channelMask;
            let mut offset = 0;
            for i in 0..6 {
                if (channelMask >> i) & 0x1 == 0 {
                    continue;
                }
                let m_Channel = &mut self.m_Channels[i];
                m_Channel.stream = s as u8;
                m_Channel.offset = offset;
                match i {
                    0 | 1 => {
                        m_Channel.format = 0;
                        m_Channel.dimension = 3;
                        break;
                    }
                    2 => {
                        m_Channel.format = 2;
                        m_Channel.dimension = 4;
                        break;
                    }
                    3 | 4 => {
                        m_Channel.format = 0;
                        m_Channel.dimension = 2;
                        break;
                    }
                    5 => {
                        m_Channel.format = 0;
                        m_Channel.dimension = 4;
                        break;
                    }
                    _ => unreachable!(),
                }
                offset += (m_Channel.dimension
                    * VertexFormat::load(m_Channel.format, version)?.get_format_size())
            }
        }
        Ok(())
    }
    fn get_streams(&mut self, version: [i32; 4]) -> Result<()> {
        let stream_count = {
            let mut max = 0;
            for i in &self.m_Channels {
                if i.stream > max {
                    max = i.stream
                }
            }
            max + 1
        };
        self.m_Streams = Vec::with_capacity(stream_count as usize);
        let mut offset = std::num::Wrapping(0);
        for s in 0..stream_count {
            let mut chnMask = 0;
            let mut stride = 0;
            for chn in 0..self.m_Channels.len() {
                let m_Channel = &self.m_Channels[chn];
                if m_Channel.stream == s {
                    if m_Channel.dimension > 0 {
                        chnMask |= 1u8 << chn;
                        stride += (m_Channel.dimension
                            * VertexFormat::load(m_Channel.format, version)?.get_format_size())
                    }
                }
            }
            self.m_Streams.push(StreamInfo {
                channelMask: chnMask,
                offset: offset.0,
                stride,
                align: 0,
                dividerOp: 0,
                frequency: 0,
            });
            offset += self.m_VertexCount * stride;
            offset = (offset + std::num::Wrapping((16u8 - 1u8) & (!(16u8 - 1u8))));
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum VertexFormat {
    Float,
    Float16,
    UNorm8,
    SNorm8,
    UNorm16,
    SNorm16,
    UInt8,
    SInt8,
    UInt16,
    SInt16,
    UInt32,
    SInt32,
}
#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum VertexChannelFormat {
    Float,
    Float16,
    Color,
    Byte,
    UInt32,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum VertexFormat2017 {
    Float,
    Float16,
    Color,
    UNorm8,
    SNorm8,
    UNorm16,
    SNorm16,
    UInt8,
    SInt8,
    UInt16,
    SInt16,
    UInt32,
    SInt32,
}
impl VertexFormat {
    fn load(format: u8, version: [i32; 4]) -> Result<Self> {
        if version[0] < 2017 {
            let result = match VertexChannelFormat::try_from(format).or(Err(Error::InvalidValue))? {
                VertexChannelFormat::Float => VertexFormat::Float,
                VertexChannelFormat::Float16 => VertexFormat::Float16,
                VertexChannelFormat::Color => VertexFormat::UNorm8,
                VertexChannelFormat::Byte => VertexFormat::UInt8,
                VertexChannelFormat::UInt32 => VertexFormat::UInt32,
            };
            return Ok(result);
        }
        if version[0] < 2019 {
            let result = match VertexFormat2017::try_from(format).or(Err(Error::InvalidValue))? {
                VertexFormat2017::Float => VertexFormat::Float,
                VertexFormat2017::Float16 => VertexFormat::Float16,
                VertexFormat2017::Color => VertexFormat::UNorm8,
                VertexFormat2017::UNorm8 => VertexFormat::UNorm8,
                VertexFormat2017::SNorm8 => VertexFormat::SNorm8,
                VertexFormat2017::UNorm16 => VertexFormat::UNorm16,
                VertexFormat2017::SNorm16 => VertexFormat::SNorm16,
                VertexFormat2017::UInt8 => VertexFormat::UInt8,
                VertexFormat2017::SInt8 => VertexFormat::SInt8,
                VertexFormat2017::UInt16 => VertexFormat::UInt16,
                VertexFormat2017::SInt16 => VertexFormat::SInt16,
                VertexFormat2017::UInt32 => VertexFormat::UInt32,
                VertexFormat2017::SInt32 => VertexFormat::SInt32,
            };
            return Ok(result);
        }
        Ok(VertexFormat::try_from(format).or(Err(Error::InvalidValue))?)
    }

    fn get_format_size(&self) -> u8 {
        match *self {
            VertexFormat::Float => 4,
            VertexFormat::UInt32 => 4,
            VertexFormat::SInt32 => 4,

            VertexFormat::Float16 => 2,
            VertexFormat::UNorm16 => 2,
            VertexFormat::SNorm16 => 2,
            VertexFormat::UInt16 => 2,
            VertexFormat::SInt16 => 2,

            VertexFormat::UNorm8 => 1,
            VertexFormat::SNorm8 => 1,
            VertexFormat::UInt8 => 1,
            VertexFormat::SInt8 => 1,
        }
    }
}
