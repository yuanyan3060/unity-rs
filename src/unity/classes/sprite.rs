#![allow(unused)]
#![allow(non_snake_case)]
use std::{borrow::Cow, collections::HashMap};

use bytes::Bytes;
use image::RgbaImage;

use crate::unity::{object::ObjectInfo, Error, FromObject, Object, Reader, Result};

use super::{
    image_alpha_merge,
    mesh::{SubMesh, VertexData},
    pptr::PPtr,
    SpriteAtlas, TextAsset, Texture2D,
};
use crate::unity::math::{RectF32, Vector2, Vector3, Vector4};

#[derive(Default, Debug)]
pub struct SecondarySpriteTexture {
    pub texture: PPtr<Texture2D>,
    pub name: String,
}

impl SecondarySpriteTexture {
    pub fn load(object: &Object, r: &mut Reader) -> Result<Self> {
        let mut result = Self::default();
        let version = object.info.version;
        result.texture = PPtr::load(object, r)?;
        result.name = r.read_string_utill_null()?;
        Ok(result)
    }
}

#[derive(Default, Debug)]
pub struct SpriteVertex {
    pub pos: Vector3,
    pub uv: Vector2,
}

impl SpriteVertex {
    pub fn load(object: &Object, r: &mut Reader) -> Result<Self> {
        let mut result = Self::default();
        let version = object.info.version;
        result.pos = r.read_vector3()?;
        if (version[0] < 4 || (version[0] == 4 && version[1] <= 3)) {
            result.uv = r.read_vector2()?;
        }
        Ok(result)
    }
}

#[derive(Default, Debug)]
pub struct SpriteRenderData {
    pub texture: PPtr<Texture2D>,
    pub alphaTexture: Option<PPtr<Texture2D>>,
    pub secondaryTextures: Vec<SecondarySpriteTexture>,
    pub m_SubMeshes: Vec<SubMesh>,
    pub m_IndexBuffer: Bytes,
    pub m_VertexData: VertexData,
    pub vertices: Vec<SpriteVertex>,
    pub indices: Vec<u16>
}

impl SpriteRenderData {
    pub fn load(object: &Object, r: &mut Reader) -> Result<Self> {
        let mut result = Self::default();
        let version = object.info.version;
        result.texture = PPtr::load(object, r)?;
        if (version[0] > 5 || (version[0] == 5 && version[1] >= 2)) {
            result.alphaTexture = Some(PPtr::load(object, r)?);
        }
        if version[0] >= 2019 {
            let size = r.read_i32()?;
            for _ in 0..size {
                result
                    .secondaryTextures
                    .push(SecondarySpriteTexture::load(&object, r)?)
            }
        }
        if (version[0] > 5 || (version[0] == 5 && version[1] >= 6)) {
            let size = r.read_i32()?;
            for _ in 0..size {
                result.m_SubMeshes.push(SubMesh::load(&object, r)?)
            }
            let size = r.read_i32()?;
            result.m_IndexBuffer = r.read_u8_list(size as usize)?;
            r.align(4)?;
            result.m_VertexData = VertexData::load(object, r)?;
        } else {
            let size = r.read_i32()?;
            for _ in 0..size {
                result.vertices.push(SpriteVertex::load(object, r)?)
            }
            let size = r.read_i32()?;
            result.indices = r.read_u16_list(size as usize)?;
            r.align(4)?;
        }
        Ok(result)
    }
}
#[derive(Default, Debug)]
pub struct Sprite {
    pub name: String,
    pub rect: RectF32,
    pub offset: Vector2,
    pub border: Option<Vector4>,
    pub pixels_to_units: f32,
    pub pivot: Vector2,
    pub extrude: u8,
    pub is_polygon: bool,
    pub render_data_key: ([u8; 16], i64),
    pub atlas_tags: Vec<String>,
    pub sprite_atlas: PPtr<SpriteAtlas>,
    pub rd: SpriteRenderData,
}
impl Sprite {
    fn new() -> Self {
        Self {
            pivot: Vector2 { x: 0.5, y: 0.5 },
            ..Default::default()
        }
    }
}
impl FromObject for Sprite {
    fn load(object: &Object) -> Result<Self> {
        let version = object.info.version;
        let mut r = object.info.reader.clone();
        r.set_offset(object.info.bytes_start as usize)?;
        let mut result = Self::new();
        result.name = r.read_aligned_string()?;
        result.rect = r.read_rect_f32()?;
        result.offset = r.read_vector2()?;
        if (version[0] > 4 || (version[0] == 4 && version[1] >= 5)) {
            result.border = Some(r.read_vector4()?);
        }
        result.pixels_to_units = r.read_f32()?;
        if (version[0] > 5
            || (version[0] == 5 && version[1] > 4)
            || (version[0] == 5 && version[1] == 4 && version[2] >= 2)
            || (version[0] == 5
                && version[1] == 4
                && version[2] == 1
                && object.info.build_type == "p"
                && version[3] >= 3))
        {
            result.pivot = r.read_vector2()?;
        }
        result.extrude = r.read_u32()? as u8;
        if (version[0] > 5 || (version[0] == 5 && version[1] >= 3)) {
            result.is_polygon = r.read_bool()?;
            r.align(4);
        }
        if (version[0] >= 2017)
        //2017 and up
        {
            let first = r.read_u8_array()?;
            let second = r.read_i64()?;
            result.render_data_key = (first, second);
            result.atlas_tags = r.read_string_list()?;
            result.sprite_atlas = PPtr::load(&object, &mut r)?;
        }
        result.rd = SpriteRenderData::load(object, &mut r)?;
        Ok(result)
    }
}

impl Sprite {
    pub fn decode_image(&self) -> Result<RgbaImage> {
        let texture = self.rd.texture.get_obj()?.decode_image()?;
        todo!()
        
    }
}