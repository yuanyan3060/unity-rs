#![allow(unused)]
#![allow(non_camel_case_types)]
use crate::unity::{object::ObjectInfo, AssetBundle, Error, FromObject, Reader, Result, Object};
use bytes::Bytes;
use image::{ImageBuffer, Rgba, RgbaImage};
use num_enum::FromPrimitive;

use super::decode::{decode_etc1, decode_etc2, decode_etc2a8};
#[derive(Debug, Eq, PartialEq, FromPrimitive, Clone, Copy)]
#[repr(i32)]
pub enum TextureFormat {
    #[num_enum(default)]
    UnknownType = -1,
    Alpha8 = 1,
    ARGB4444,
    RGB24,
    RGBA32,
    ARGB32,
    RGB565 = 7,
    R16 = 9,
    DXT1,
    DXT5 = 12,
    RGBA4444,
    BGRA32,
    RHalf,
    RGHalf,
    RGBAHalf,
    RFloat,
    RGFloat,
    RGBAFloat,
    YUY2,
    RGB9e5Float,
    BC4 = 26,
    BC5,
    BC6H = 24,
    BC7,
    DXT1Crunched = 28,
    DXT5Crunched,
    PVRTC_RGB2,
    PVRTC_RGBA2,
    PVRTC_RGB4,
    PVRTC_RGBA4,
    ETC_RGB4,
    ATC_RGB4,
    ATC_RGBA8,
    EAC_R = 41,
    EAC_R_SIGNED,
    EAC_RG,
    EAC_RG_SIGNED,
    ETC2_RGB,
    ETC2_RGBA1,
    ETC2_RGBA8,
    ASTC_RGB_4x4,
    ASTC_RGB_5x5,
    ASTC_RGB_6x6,
    ASTC_RGB_8x8,
    ASTC_RGB_10x10,
    ASTC_RGB_12x12,
    ASTC_RGBA_4x4,
    ASTC_RGBA_5x5,
    ASTC_RGBA_6x6,
    ASTC_RGBA_8x8,
    ASTC_RGBA_10x10,
    ASTC_RGBA_12x12,
    ETC_RGB4_3DS,
    ETC_RGBA8_3DS,
    RG16,
    R8,
    ETC_RGB4Crunched,
    ETC2_RGBA8Crunched,
    ASTC_HDR_4x4,
    ASTC_HDR_5x5,
    ASTC_HDR_6x6,
    ASTC_HDR_8x8,
    ASTC_HDR_10x10,
    ASTC_HDR_12x12,
}

impl Default for TextureFormat {
    fn default() -> Self {
        Self::UnknownType
    }
}
#[derive(Default)]
pub struct GLTextureSettings {
    filter_mode: i32,
    aniso: i32,
    mip_bias: f32,
    wrap_mode: i32,
}

impl GLTextureSettings {
    pub fn load(object_info: &ObjectInfo, r: &mut Reader) -> Result<Self> {
        let mut result = Self::default();
        result.filter_mode = r.read_i32()?;
        result.aniso = r.read_i32()?;
        result.mip_bias = r.read_f32()?;
        if object_info.version[0] >= 2017 {
            result.wrap_mode = r.read_i32()?;
            let wrap_w = r.read_i32()?;
            let wrap_h = r.read_i32()?;
        } else {
            result.wrap_mode = r.read_i32()?;
        }
        Ok(result)
    }
}

#[derive(Default)]
pub struct StreamingInfo {
    offset: u64,
    size: u32,
    path: String,
}

impl StreamingInfo {
    pub fn load(object_info: &ObjectInfo, r: &mut Reader) -> Result<Self> {
        let mut result = Self::default();
        if object_info.version[0] >= 2020 {
            result.offset = r.read_u64()?;
        } else {
            result.offset = r.read_u32()? as u64;
        }
        result.size = r.read_u32()?;
        result.path = r.read_aligned_string()?;
        Ok(result)
    }
}
#[derive(Default)]
pub struct Texture2D {
    pub name: String,
    pub forced_fallback_format: i32,
    pub downscale_fallback: bool,
    pub width: i32,
    pub height: i32,
    pub complete_image_size: i32,
    pub format: TextureFormat,
    pub mip_map: bool,
    pub mip_count: i32,
    pub is_read_able: bool,
    pub image_count: i32,
    pub texture_dimension: i32,
    pub light_map_format: i32,
    pub color_space: i32,
    pub size: i32,
    pub stream_info: StreamingInfo,
    pub texture_setting: GLTextureSettings,
    pub data: Bytes,
}
impl FromObject for Texture2D {
    fn load(object: &Object) -> Result<Self> {
        let mut r = object.info.reader.clone();
        r.set_offset(object.info.bytes_start as usize)?;
        let mut result = Self::default();
        result.name = r.read_aligned_string()?;
        let version = &object.info.version;
        if version[0] > 2017 || (version[0] == 2017 && version[1] >= 3) {
            result.forced_fallback_format = r.read_i32()?;
            result.downscale_fallback = r.read_bool()?;
            if version[0] > 2020 || (version[0] == 2020 && version[1] >= 2) {
                let is_alpha_channel_optional = r.read_bool()?;
            }
            r.align(4)?;
        }
        result.width = r.read_i32()?;
        result.height = r.read_i32()?;
        result.complete_image_size = r.read_i32()?;
        if object.info.version[0] >= 2020 {
            let mips_stripped = r.read_i32()?;
        }
        result.format = TextureFormat::from(r.read_i32()?);
        let mut mip_map = false;
        if object.info.version[0] < 5 {
            mip_map = r.read_bool()?;
        } else if object.info.version[0] == 5 && object.info.version[1] < 2 {
            mip_map = r.read_bool()?;
        } else {
            result.mip_count = r.read_i32()?;
        }
        if version[0] > 2 || (version[0] == 2 && version[1] >= 6) {
            result.is_read_able = r.read_bool()?;
        }
        if version[0] >= 2020 {
            let is_pre_processed = r.read_bool()?;
        }
        if version[0] > 2019 || (version[0] == 2019 && version[1] >= 3) {
            let is_ignore_master_texture_limit = r.read_bool()?;
        }
        if version[0] >= 3 {
            if version[0] < 5 || (version[0] == 5 && version[1] <= 4) {
                let read_allowed = r.read_bool()?;
            }
        }
        if version[0] > 2018 || (version[0] == 2018 && version[1] >= 2) {
            let streaming_mip_maps = r.read_bool()?;
        }
        r.align(4)?;
        if version[0] > 2018 || (version[0] == 2018 && version[1] >= 2) {
            let streaming_mip_maps_priority = r.read_i32()?;
        }
        result.image_count = r.read_i32()?;
        result.texture_dimension = r.read_i32()?;
        result.texture_setting = GLTextureSettings::load(&object.info, &mut r)?;
        if version[0] >= 3 {
            result.light_map_format = r.read_i32()?;
        }
        if version[0] > 3 || (version[0] == 3 && version[1] >= 5) {
            result.color_space = r.read_i32()?;
        }
        if version[0] > 2020 || (version[0] == 2020 && version[1] >= 2) {
            let length = r.read_i32()?;
            let platform_blob = r.read_u8_slice(length as usize)?;
            r.align(4)?;
        }
        result.size = r.read_i32()?;
        if result.size == 0 && ((version[0] == 5 && version[1] >= 3) || version[0] > 5) {
            result.stream_info = StreamingInfo::load(&object.info, &mut r)?;
        }
        if result.stream_info.path.is_empty() {
            result.data = r.read_u8_list(result.size as usize)?;
        } else {
            let path = result
                .stream_info
                .path
                .split("/")
                .last()
                .ok_or(Error::InvalidValue)?;
            for i in 0..object.info.bundle.nodes.len() {
                let node = &object.info.bundle.nodes[i];
                if node.path != path {
                    continue;
                }
                let file = &object.info.bundle.files[i];
                let mut r = Reader::new(file.clone(), crate::unity::ByteOrder::Big);
                r.set_offset(result.stream_info.offset as usize)?;
                result.data = r.read_u8_list(result.stream_info.size as usize)?;
            }
        }
        Ok(result)
    }
}
impl Texture2D {
    pub fn decode_image(&self) -> Result<RgbaImage> {
        let width = self.width;
        let height = self.height;
        let format = self.format;
        match format {
            TextureFormat::ETC2_RGBA8 => {
                let mut result: ImageBuffer<Rgba<u8>, Vec<u8>> =
                    ImageBuffer::new(width as u32, height as u32);
                decode_etc2a8(&self.data, width as usize, height as usize, result.as_mut());
                return Ok(result.into());
            }
            TextureFormat::ETC2_RGB => {
                let mut result: ImageBuffer<Rgba<u8>, Vec<u8>> =
                    ImageBuffer::new(width as u32, height as u32);
                decode_etc2(&self.data, width as usize, height as usize, result.as_mut());
                return Ok(result.into());
            }
            TextureFormat::ETC_RGB4 => {
                let mut result: ImageBuffer<Rgba<u8>, Vec<u8>> =
                    ImageBuffer::new(width as u32, height as u32);
                decode_etc1(&self.data, width as usize, height as usize, result.as_mut());
                return Ok(result.into());
            }
            _ => return Err(Error::Unimplemented),
        }
    }
}
