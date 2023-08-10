use crate::classes::pptr::PPtr;
use crate::classes::sprite::{SecondarySpriteTexture, SpriteSettings};
use crate::classes::{FromObject, Sprite, Texture2D};
use crate::env::Object;
use crate::error::UnityResult;
use crate::math::{RectF32, Vector2, Vector4};
use crate::reader::Reader;
use std::collections::HashMap;

pub struct SpriteAtlasData<'a> {
    pub texture: PPtr<'a, Texture2D>,
    pub alpha_texture: PPtr<'a, Texture2D>,
    pub texture_rect: RectF32,
    pub texture_rect_offset: Vector2,
    pub atlas_rect_offset: Vector2,
    pub uv_transform: Vector4,
    pub downscale_multiplier: f32,
    pub settings_raw: SpriteSettings,
    pub secondary_textures: Vec<SecondarySpriteTexture<'a>>,
}

impl<'a> SpriteAtlasData<'a> {
    pub fn load(object: &'a Object, r: &mut Reader) -> UnityResult<Self> {
        let version = object.info.version;
        let texture = PPtr::load(object, r)?;
        let alpha_texture = PPtr::load(object, r)?;
        let texture_rect = r.read_rect_f32()?;
        let texture_rect_offset = r.read_vector2()?;
        let atlas_rect_offset = if version[0] > 2017 || (version[0] == 2017 && version[1] >= 2) { r.read_vector2()? } else { Vector2::default() };
        let uv_transform = r.read_vector4()?;
        let downscale_multiplier = r.read_f32()?;
        let settings_raw = SpriteSettings::load(&object.info, r)?;
        let mut secondary_textures = Vec::new();
        if version[0] > 2020 || (version[0] == 2020 && version[1] >= 2) {
            for _ in 0..r.read_i32()? {
                secondary_textures.push(SecondarySpriteTexture::load(object, r)?)
            }
            r.align(4)?;
        }
        Ok(Self {
            texture,
            alpha_texture,
            texture_rect,
            texture_rect_offset,
            atlas_rect_offset,
            uv_transform,
            downscale_multiplier,
            settings_raw,
            secondary_textures,
        })
    }
}
pub struct SpriteAtlas<'a> {
    pub name: String,
    pub packed_sprites: Vec<PPtr<'a, Sprite<'a>>>,
    pub render_data_map: HashMap<([u8; 16], i64), SpriteAtlasData<'a>>,
    pub is_variant: bool,
}
impl<'a> FromObject<'a> for SpriteAtlas<'a> {
    fn load(object: &'a Object) -> UnityResult<Self> {
        let mut r = object.info.get_reader();
        let name = r.read_aligned_string()?;
        let mut packed_sprites = Vec::new();
        for _ in 0..r.read_i32()? {
            packed_sprites.push(PPtr::load(object, &mut r)?);
        }
        let _packed_sprite_names_to_index = r.read_string_list()?;
        let render_data_map_size = r.read_i32()?;
        let mut render_data_map = HashMap::new();
        for _ in 0..render_data_map_size {
            let first = r.read_u8_array::<16>()?;
            let second = r.read_i64()?;
            let value = SpriteAtlasData::load(object, &mut r)?;
            render_data_map.insert((first, second), value);
        }
        let _tag = r.read_aligned_string()?;
        let is_variant = r.read_bool()?;
        Ok(Self {
            name,
            packed_sprites,
            render_data_map,
            is_variant,
        })
    }
}
