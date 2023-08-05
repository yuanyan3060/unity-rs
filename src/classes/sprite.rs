use crate::classes::mesh::{SubMesh, VertexData};
use crate::classes::pptr::PPtr;
use crate::classes::sprite_atlas::SpriteAtlas;
use crate::classes::{FromObject, Texture2D};
use crate::env::Object;
use crate::error::UnityResult;
use crate::math::{Matrix4x4, RectF32, Vector2, Vector3, Vector4};
use crate::object::ObjectInfo;
use crate::reader::{ByteOrder, Reader};
use image::imageops::FilterType;
use image::DynamicImage;
use imageproc::point::Point;
use std::borrow::Cow;

use super::mesh::BoneWeights4;

#[derive(Debug)]
pub struct SecondarySpriteTexture<'a> {
    pub texture: PPtr<'a, Texture2D>,
    pub name: String,
}

impl<'a> SecondarySpriteTexture<'a> {
    pub fn load(object: &'a Object, r: &mut Reader) -> UnityResult<Self> {
        let texture = PPtr::load(object, r)?;
        let name = r.read_string_util_null()?;
        Ok(Self { texture, name })
    }
}

#[derive(Default, Debug)]
pub struct SpriteVertex {
    pub pos: Vector3,
    pub uv: Vector2,
}

impl SpriteVertex {
    pub fn load(object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let mut result = Self::default();
        let version = object.version;
        result.pos = r.read_vector3()?;
        if version[0] < 4 || (version[0] == 4 && version[1] <= 3) {
            result.uv = r.read_vector2()?;
        }
        Ok(result)
    }
}

#[derive(Debug)]
pub struct SpriteRenderData<'a> {
    pub texture: PPtr<'a, Texture2D>,
    pub alpha_texture: Option<PPtr<'a, Texture2D>>,
    pub secondary_textures: Vec<SecondarySpriteTexture<'a>>,
    pub sub_meshes: Vec<SubMesh>,
    pub index_buffer: Vec<u8>,
    pub vertex_data: VertexData,
    pub vertices: Vec<SpriteVertex>,
    pub indices: Vec<u16>,
    pub bindpose: Vec<Matrix4x4>,
    pub source_skin: Vec<BoneWeights4>,
    pub texture_rect: RectF32,
    pub texture_rect_offset: Vector2,
    pub atlas_rect_offset: Vector2,
    pub setting_raw: SpriteSettings,
    pub uv_transform: Vector4,
    pub downscale_multiplier: f32,
}

impl<'a> SpriteRenderData<'a> {
    pub fn load(object: &'a Object, r: &mut Reader) -> UnityResult<Self> {
        let version = object.info.version;
        let mut result = Self {
            texture: PPtr::load(object, r)?,
            alpha_texture: None,
            secondary_textures: Vec::new(),
            sub_meshes: Vec::new(),
            index_buffer: Vec::new(),
            vertex_data: VertexData::default(),
            vertices: Vec::new(),
            indices: Vec::new(),
            bindpose: Vec::new(),
            source_skin: Vec::new(),
            texture_rect: RectF32::default(),
            texture_rect_offset: Vector2::default(),
            atlas_rect_offset: Vector2::default(),
            setting_raw: SpriteSettings::default(),
            uv_transform: Vector4::default(),
            downscale_multiplier: 1.0,
        };
        if version[0] > 5 || (version[0] == 5 && version[1] >= 2) {
            result.alpha_texture = Some(PPtr::load(object, r)?);
        }
        if version[0] >= 2019 {
            let size = r.read_i32()?;
            for _ in 0..size {
                result.secondary_textures.push(SecondarySpriteTexture::load(&object, r)?)
            }
        }
        if version[0] > 5 || (version[0] == 5 && version[1] >= 6) {
            let size = r.read_i32()?;
            for _ in 0..size {
                result.sub_meshes.push(SubMesh::load(&object.info, r)?)
            }
            let size = r.read_i32()?;
            result.index_buffer = r.read_u8_list(size as usize)?;
            r.align(4)?;
            result.vertex_data = VertexData::load(&object.info, r)?;
        } else {
            let size = r.read_i32()?;
            for _ in 0..size {
                result.vertices.push(SpriteVertex::load(&object.info, r)?)
            }
            let size = r.read_i32()?;
            result.indices = r.read_u16_list(size as usize)?;
            r.align(4)?;
        }
        if version[0] > 2018 {
            let size = r.read_i32()?;
            result.bindpose = r.read_matrix4x4_list(size as usize)?;
            if version[0] == 2018 && version[1] < 2 {
                let size = r.read_i32()? as usize;
                result.source_skin = Vec::with_capacity(size);
                for _ in 0..size {
                    result.source_skin.push(BoneWeights4::load(&object.info, r)?);
                }
            }
        }
        result.texture_rect = r.read_rect_f32()?;
        result.texture_rect_offset = r.read_vector2()?;
        if version[0] > 5 || (version[0] == 5 && version[1] >= 6) {
            result.atlas_rect_offset = r.read_vector2()?;
        }
        result.setting_raw = SpriteSettings::load(&object.info, r)?;
        if version[0] > 4 || (version[0] == 4 && version[1] >= 5) {
            result.uv_transform = r.read_vector4()?;
        }
        if version[0] >= 2017 {
            result.downscale_multiplier = r.read_f32()?;
        }
        Ok(result)
    }

    pub fn get_triangles(&self) -> UnityResult<Vec<[Vector2; 3]>> {
        let mut result = Vec::new();
        if !self.vertices.is_empty() {
            let vertices: Vec<_> = self.vertices.iter().map(|i| Vector2 { x: i.pos.x, y: i.pos.y }).collect();
            let triangle_count = self.indices.len() / 3;
            for i in 0..triangle_count {
                let first = self.indices[3 * i] as usize;
                let second = self.indices[3 * i + 1] as usize;
                let third = self.indices[3 * i + 2] as usize;
                result.push([vertices[first], vertices[second], vertices[third]])
            }
        } else {
            let channel = &self.vertex_data.channels[0];
            let stream = &self.vertex_data.streams[channel.stream as usize];
            let mut vertex_r = Reader::new(&self.vertex_data.data_size, ByteOrder::Little);
            let mut index_r = Reader::new(&self.index_buffer, ByteOrder::Little);
            for sub_mesh in &self.sub_meshes {
                let mut offset = stream.offset as usize;
                offset += sub_mesh.first_vertex as usize * stream.stride as usize;
                offset += channel.offset as usize;
                vertex_r.set_offset(offset)?;
                let mut vertices = Vec::with_capacity(sub_mesh.vertex_count as usize);
                for _ in 0..sub_mesh.vertex_count {
                    let v3 = vertex_r.read_vector3()?;
                    vertices.push(Vector2 { x: v3.x, y: v3.y });
                    let offset = vertex_r.get_offset();
                    vertex_r.set_offset(offset + stream.stride as usize - 12)?;
                }
                index_r.set_offset(sub_mesh.first_bytes as usize)?;
                let triangle_count = sub_mesh.index_count as usize / 3;
                for _ in 0..triangle_count {
                    let first = index_r.read_u16()? - sub_mesh.first_bytes as u16;
                    let second = index_r.read_u16()? - sub_mesh.first_bytes as u16;
                    let third = index_r.read_u16()? - sub_mesh.first_bytes as u16;
                    result.push([vertices[first as usize], vertices[second as usize], vertices[third as usize]])
                }
            }
        }
        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub enum SpritePackingMode {
    Tight = 0,
    Rectangle = 1,
}

impl Default for SpritePackingMode {
    fn default() -> Self {
        Self::Tight
    }
}

#[derive(Clone, Debug)]
pub enum SpritePackingRotation {
    None = 0,
    FlipHorizontal = 1,
    FlipVertical = 2,
    Rotate180 = 3,
    Rotate90 = 4,
}

impl Default for SpritePackingRotation {
    fn default() -> Self {
        Self::None
    }
}
#[derive(Clone, Debug)]
pub enum SpriteMeshType {
    FullRect = 0,
    Tight = 1,
}

impl Default for SpriteMeshType {
    fn default() -> Self {
        Self::FullRect
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct SpriteSettings {
    raw: u32,
    packed: bool,
    packing_mode: SpritePackingMode,
    packing_rotation: SpritePackingRotation,
    mesh_type: SpriteMeshType,
}

impl SpriteSettings {
    pub fn load(_object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let raw = r.read_u32()?;
        let packed = raw & 1 == 1;
        let packing_mode = match (raw >> 1) & 1 {
            0 => SpritePackingMode::Tight,
            1 => SpritePackingMode::Rectangle,
            _ => unreachable!(),
        };
        let packing_rotation = match (raw >> 2) & 0xf {
            0 => SpritePackingRotation::None,
            1 => SpritePackingRotation::FlipHorizontal,
            2 => SpritePackingRotation::FlipVertical,
            3 => SpritePackingRotation::Rotate180,
            4 => SpritePackingRotation::Rotate90,
            _ => unreachable!(),
        };
        let mesh_type = match (raw >> 6) & 1 {
            0 => SpriteMeshType::FullRect,
            1 => SpriteMeshType::Tight,
            _ => unreachable!(),
        };
        Ok(Self {
            raw,
            packed,
            packing_mode,
            packing_rotation,
            mesh_type,
        })
    }
}

#[derive(Debug)]
pub struct Sprite<'a> {
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
    pub sprite_atlas: Option<PPtr<'a, SpriteAtlas<'a>>>,
    pub rd: SpriteRenderData<'a>,
}

impl<'a> FromObject<'a> for Sprite<'a> {
    fn load(object: &'a Object) -> UnityResult<Self> {
        let version = object.info.version;
        let name: String;
        let rect: RectF32;
        let offset: Vector2;
        let mut border: Option<Vector4> = None;
        let pixels_to_units: f32;
        let mut pivot: Vector2 = Vector2 { x: 0.5, y: 0.5 };
        let extrude: u8;
        let mut is_polygon: bool = false;
        let mut render_data_key: ([u8; 16], i64) = ([0u8; 16], 0);
        let mut atlas_tags: Vec<String> = Vec::new();
        let mut sprite_atlas: Option<PPtr<SpriteAtlas>> = None;
        let rd: SpriteRenderData;
        let mut r = object.info.get_reader();
        name = r.read_aligned_string()?;
        rect = r.read_rect_f32()?;
        offset = r.read_vector2()?;
        if version[0] > 4 || (version[0] == 4 && version[1] >= 5) {
            border = Some(r.read_vector4()?);
        }
        pixels_to_units = r.read_f32()?;
        if version[0] > 5
            || (version[0] == 5 && version[1] > 4)
            || (version[0] == 5 && version[1] == 4 && version[2] >= 2)
            || (version[0] == 5 && version[1] == 4 && version[2] == 1 && object.info.build_type == "p" && version[3] >= 3)
        {
            pivot = r.read_vector2()?;
        }
        extrude = r.read_u32()? as u8;
        if version[0] > 5 || (version[0] == 5 && version[1] >= 3) {
            is_polygon = r.read_bool()?;
            r.align(4)?;
        }
        if version[0] >= 2017
        //2017 and up
        {
            let first = r.read_u8_array()?;
            let second = r.read_i64()?;
            render_data_key = (first, second);
            atlas_tags = r.read_string_list()?;
            sprite_atlas = Some(PPtr::load(&object, &mut r)?);
        }
        rd = SpriteRenderData::load(object, &mut r)?;
        Ok(Self {
            name,
            rect,
            offset,
            border,
            pixels_to_units,
            pivot,
            extrude,
            is_polygon,
            render_data_key,
            atlas_tags,
            sprite_atlas,
            rd,
        })
    }
}

impl<'a> Sprite<'a> {
    pub fn decode_image(&self) -> UnityResult<DynamicImage> {
        if let Some(sprite_atlas) = self.sprite_atlas.as_ref().and_then(|x| x.get_obj()) {
            if let Some(sprite_atlas_data) = sprite_atlas.read::<SpriteAtlas>()?.render_data_map.get(&self.render_data_key) {
                if let Some(texture2d) = sprite_atlas_data.texture.get_obj() {
                    let texture2d = texture2d.read()?;
                    let rect = sprite_atlas_data.texture_rect;
                    let offset = sprite_atlas_data.texture_rect_offset;
                    let downscale_multiplier = sprite_atlas_data.downscale_multiplier;
                    let setting = sprite_atlas_data.settings_raw.clone();
                    return self.cut_image(&texture2d, rect, offset, downscale_multiplier, &setting);
                }
            }
        }
        if let Some(texture2d) = self.rd.texture.get_obj() {
            let texture2d = texture2d.read()?;
            return self.cut_image(&texture2d, self.rd.texture_rect, self.rd.texture_rect_offset, self.rd.downscale_multiplier, &self.rd.setting_raw);
        }
        todo!()
    }
    fn cut_image(&self, texture2d: &Texture2D, rect: RectF32, _offset: Vector2, downscale_multiplier: f32, setting: &SpriteSettings) -> UnityResult<DynamicImage> {
        let origin_image = texture2d.decode_image()?;
        let mut origin_image = Cow::Borrowed(&*origin_image);
        if downscale_multiplier > 0.0 && downscale_multiplier != 1.0 {
            let w = (texture2d.width as f32) / downscale_multiplier;
            let h = (texture2d.height as f32) / downscale_multiplier;
            origin_image = Cow::Owned(origin_image.resize(w as u32, h as u32, FilterType::Nearest));
        }
        let rect_x = rect.x.floor() as u32;
        let rect_y = rect.y.floor() as u32;
        let rect_w = rect.w.floor() as u32;
        let rect_h = rect.h.floor() as u32;
        let rect_w = rect_w.min(origin_image.width());
        let rect_h = rect_h.min(origin_image.height());
        let mut sprite_image = origin_image.as_ref().crop_imm(rect_x, origin_image.height() - rect_y - rect_h, rect_w, rect_h);
        if setting.packed {
            match setting.packing_rotation {
                SpritePackingRotation::None => (),
                SpritePackingRotation::FlipHorizontal => image::imageops::flip_horizontal_in_place(&mut sprite_image),
                SpritePackingRotation::FlipVertical => image::imageops::flip_vertical_in_place(&mut sprite_image),
                SpritePackingRotation::Rotate180 => image::imageops::rotate180_in_place(&mut sprite_image),
                SpritePackingRotation::Rotate90 => sprite_image = sprite_image.rotate270(),
            }
        }
        if let SpritePackingMode::Tight = setting.packing_mode {
            let mut points = self.rd.get_triangles()?;
            let mut min_x: Option<f32> = None;
            let mut min_y: Option<f32> = None;
            for i in &points {
                for j in i {
                    match min_x {
                        Some(m) => min_x = Some(m.min(j.x)),
                        None => min_x = Some(j.x),
                    }
                    match min_y {
                        Some(m) => min_y = Some(m.min(j.y)),
                        None => min_y = Some(j.y),
                    }
                }
            }
            let min_x = min_x.unwrap_or_default();
            let min_y = min_y.unwrap_or_default();
            for i in &mut points {
                for j in i {
                    j.x = (j.x - min_x) * self.pixels_to_units;
                    j.y = (j.y - min_y) * self.pixels_to_units;
                }
            }
            let mut mask = image::GrayImage::from_pixel(sprite_image.width(), sprite_image.height(), image::Luma([0]));
            for i in points {
                let poly = [
                    Point::new(i[0].x.round() as i32, rect_h as i32 - i[0].y.round() as i32),
                    Point::new(i[1].x.round() as i32, rect_h as i32 - i[1].y.round() as i32),
                    Point::new(i[2].x.round() as i32, rect_h as i32 - i[2].y.round() as i32),
                ];
                if poly[0] == poly[2] {
                    continue;
                }
                imageproc::drawing::draw_polygon_mut(&mut mask, &poly, image::Luma([255]))
            }
            let img = imageproc::map::map_colors2(&sprite_image, &mask, |a, b| {
                if b.0[0] != 0 {
                    image::Rgba([a.0[0], a.0[1], a.0[2], a.0[3]])
                } else {
                    image::Rgba([0, 0, 0, 0])
                }
            });
            return Ok(img.into());
        }
        return Ok(sprite_image);
    }
}
