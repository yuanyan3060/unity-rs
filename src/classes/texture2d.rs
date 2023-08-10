#![allow(dead_code, non_upper_case_globals)]
use crate::classes::FromObject;
use crate::env::Object;
use crate::error::{UnityError, UnityResult};
use crate::object::ObjectInfo;
use crate::reader::{ByteOrder, Reader};
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use image::{ImageBuffer, Rgba, RgbaImage};
use num_enum::FromPrimitive;
use std::sync::Arc;
use texture_decoder::{ImageSize, Texture2DDecoder};
use texture_decoder::implments::{Alpha8, ARGB32, ARGB4444, BGRA32, R16, RGB24, RGB565, RGBA32, RGBA4444};

#[allow(non_camel_case_types, non_upper_case_globals)]
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
    pub fn load(object_info: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let mut result = Self::default();
        result.filter_mode = r.read_i32()?;
        result.aniso = r.read_i32()?;
        result.mip_bias = r.read_f32()?;
        if object_info.version[0] >= 2017 {
            result.wrap_mode = r.read_i32()?;
            let _wrap_w = r.read_i32()?;
            let _wrap_h = r.read_i32()?;
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
    pub fn load(object_info: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
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
    cache: Arc<DashMap<i64, RgbaImage>>,
    pub path_id: i64,
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
    pub data: Vec<u8>,
}

impl<'a> FromObject<'a> for Texture2D {
    fn load(object: &Object) -> UnityResult<Self> {
        let mut r = object.info.get_reader();
        let mut result = Self::default();
        result.cache = object.cache.clone();
        result.path_id = object.info.path_id;
        result.name = r.read_aligned_string()?;
        let version = &object.info.version;
        if version[0] > 2017 || (version[0] == 2017 && version[1] >= 3) {
            result.forced_fallback_format = r.read_i32()?;
            result.downscale_fallback = r.read_bool()?;
            if version[0] > 2020 || (version[0] == 2020 && version[1] >= 2) {
                let _is_alpha_channel_optional = r.read_bool()?;
            }
            r.align(4)?;
        }
        result.width = r.read_i32()?;
        result.height = r.read_i32()?;
        result.complete_image_size = r.read_i32()?;
        if object.info.version[0] >= 2020 {
            let _mips_stripped = r.read_i32()?;
        }
        result.format = TextureFormat::from(r.read_i32()?);
        let mut _mip_map = false;
        if object.info.version[0] < 5 {
            _mip_map = r.read_bool()?;
        } else if object.info.version[0] == 5 && object.info.version[1] < 2 {
            _mip_map = r.read_bool()?;
        } else {
            result.mip_count = r.read_i32()?;
        }
        if version[0] > 2 || (version[0] == 2 && version[1] >= 6) {
            result.is_read_able = r.read_bool()?;
        }
        if version[0] >= 2020 {
            let _is_pre_processed = r.read_bool()?;
        }
        if version[0] > 2019 || (version[0] == 2019 && version[1] >= 3) {
            let _is_ignore_master_texture_limit = r.read_bool()?;
        }
        if version[0] >= 3 {
            if version[0] < 5 || (version[0] == 5 && version[1] <= 4) {
                let _read_allowed = r.read_bool()?;
            }
        }
        if version[0] > 2018 || (version[0] == 2018 && version[1] >= 2) {
            let _streaming_mip_maps = r.read_bool()?;
        }
        r.align(4)?;
        if version[0] > 2018 || (version[0] == 2018 && version[1] >= 2) {
            let _streaming_mip_maps_priority = r.read_i32()?;
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
            let _platform_blob = r.read_u8_slice(length as usize)?;
            r.align(4)?;
        }
        result.size = r.read_i32()?;
        if result.size == 0 && ((version[0] == 5 && version[1] >= 3) || version[0] > 5) {
            result.stream_info = StreamingInfo::load(&object.info, &mut r)?;
        }
        if result.stream_info.path.is_empty() {
            result.data = r.read_u8_list(result.size as usize)?;
        } else {
            let path = result.stream_info.path.split("/").last().ok_or(UnityError::InvalidValue)?;
            for i in 0..object.bundle.nodes.len() {
                let node = &object.bundle.nodes[i];
                if node.path != path {
                    continue;
                }
                let file = &object.bundle.files[i];
                let mut r = Reader::new(file.as_slice(), ByteOrder::Big);
                r.set_offset(result.stream_info.offset as usize)?;
                result.data = r.read_u8_list(result.stream_info.size as usize)?;
            }
        }
        Ok(result)
    }
}
impl Texture2D {
    pub fn decode_image(&self) -> UnityResult<Ref<i64, RgbaImage>> {
        if let Some(img) = self.cache.get(&self.path_id) {
            return Ok(img);
        }
        let img = self.decode_image_without_cache()?;
        self.cache.insert(self.path_id, img);
        return Ok(self.cache.get(&self.path_id).unwrap());
    }

    pub fn decode_image_without_cache(&self) -> UnityResult<RgbaImage> {
        let width = self.width;
        let height = self.height;
        let format = self.format;
        let size = ImageSize::new(width as usize,height as usize);
        let mut result: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width as u32, height as u32);
        let image = result.as_mut_ptr();
        let image = image.cast::<u32>();
        let image = unsafe { std::slice::from_raw_parts_mut(image, (width * height) as usize) };
        match format {
            TextureFormat::ETC2_RGBA8 => {
                texture2ddecoder::decode_etc2_rgba8(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            }
            TextureFormat::ETC2_RGB => {
                texture2ddecoder::decode_etc2_rgb(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            }
            TextureFormat::ETC_RGB4 => {
                texture2ddecoder::decode_etc1(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            }
            TextureFormat::ATC_RGB4 => {
                texture2ddecoder::decode_atc_rgb4(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            }
            TextureFormat::ATC_RGBA8 => {
                texture2ddecoder::decode_atc_rgba8(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            }
            TextureFormat::ASTC_RGBA_4x4 => {
                texture2ddecoder::decode_astc_4_4(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            }
            TextureFormat::ASTC_RGBA_5x5 => {
                texture2ddecoder::decode_astc_5_5(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            }
            TextureFormat::ASTC_RGBA_6x6 => {
                texture2ddecoder::decode_astc_6_6(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            }
            TextureFormat::ASTC_RGBA_8x8 => {
                texture2ddecoder::decode_astc_8_8(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            }
            TextureFormat::ASTC_RGBA_10x10 => {
                texture2ddecoder::decode_astc_10_10(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            }
            TextureFormat::ASTC_RGBA_12x12 => {
                texture2ddecoder::decode_astc_12_12(&self.data, width as usize, height as usize, image)?;
                Ok(result)
            },
            TextureFormat::ARGB32=>{
                let img = Texture2DDecoder::texture_decode_image::<ARGB32>(&size,&self.data)?;
                Ok(img)
            }
            TextureFormat::Alpha8=>{
                let img = Texture2DDecoder::texture_decode_image::<Alpha8>(&size,&self.data)?;
                Ok(img)
            }
            TextureFormat::ARGB4444=>{
                let img = Texture2DDecoder::texture_decode_image::<ARGB4444>(&size,&self.data)?;
                Ok(img)
            }TextureFormat::RGB24=>{
                let img = Texture2DDecoder::texture_decode_image::<RGB24>(&size,&self.data)?;
                Ok(img)
            }TextureFormat::RGBA32=>{
                let img = Texture2DDecoder::texture_decode_image::<RGBA32>(&size,&self.data)?;
                Ok(img)
            }TextureFormat::R16=>{
                let img = Texture2DDecoder::texture_decode_image::<R16>(&size,&self.data)?;
                Ok(img)
            }TextureFormat::RGB565=>{
                let img = Texture2DDecoder::texture_decode_image::<RGB565>(&size,&self.data)?;
                Ok(img)
            }TextureFormat::RGBA4444=>{
                let img = Texture2DDecoder::texture_decode_image::<RGBA4444>(&size,&self.data)?;
                Ok(img)
            }TextureFormat::BGRA32=>{
                let img = Texture2DDecoder::texture_decode_image::<BGRA32>(&size,&self.data)?;
                Ok(img)
            }
            _ => Err(UnityError::Unimplemented),
        }
    }
}
