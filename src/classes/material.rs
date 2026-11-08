use std::collections::HashMap;

use crate::classes::FromObject;
use crate::env::Object;
use crate::error::UnityResult;
use crate::math::{Color, Vector2};
use crate::reader::Reader;

use super::pptr::PPtr;
use super::shader::Shader;
use super::Texture2D;

pub struct Material<'a> {
    pub name: String,
    pub shader: PPtr<'a, Shader>,
    pub saved_properties: UnityPropertySheet<'a>,
}

impl<'a> FromObject<'a> for Material<'a> {
    fn load(object: &'a Object<'a>) -> UnityResult<Self> {
        let version = object.info.version;
        let r = &mut object.info.get_reader();
        let name = r.read_aligned_string()?;
        let shader = PPtr::load(object, r)?;
        if version[0] == 4 && version[1] >= 1 {
            let _shader_keywords = r.read_string_list()?;
        }
        if version[0] > 2021 || (version[0] == 2021 && version[1] >= 3) {
            let _valid_keywords = r.read_string_list()?;
            let _invalid_keywords = r.read_string_list()?;
        } else if version[0] >= 5 {
            let _shader_keywords = r.read_aligned_string()?;
        }
        if version[0] >= 5 {
            let _lightmap_flags = r.read_u32()?;
        }
        if version[0] > 5 || (version[0] == 5 && version[1] >= 6) {
            let _enable_instancing_variants = r.read_bool()?;
            r.align(4)?;
        }
        if version[0] > 4 || (version[0] == 4 && version[1] >= 3) {
            let _custom_render_queue = r.read_i32()?;
        }
        if version[0] > 5 || (version[0] == 5 && version[1] >= 1) {
            let string_tag_map_size = r.read_i32()?;
            for _ in 0..string_tag_map_size {
                let _first = r.read_aligned_string()?;
                let _second = r.read_aligned_string()?;
            }
        }
        if version[0] > 5 || (version[0] == 5 && version[1] >= 6) {
            let _disabled_shader_passes = r.read_string_list()?;
        }
        Ok(Self {
            name,
            shader,
            saved_properties: UnityPropertySheet::load(object, r)?,
        })
    }

    fn class() -> super::ClassID {
        super::ClassID::Material
    }
}

pub struct UnityPropertySheet<'a> {
    pub tex_envs: HashMap<String, UnityTexEnv<'a>>,
    pub ints: HashMap<String, i32>,
    pub floats: HashMap<String, f32>,
    pub colors: HashMap<String, Color>,
}

impl<'a> UnityPropertySheet<'a> {
    pub(super) fn load(object: &'a Object, r: &mut Reader) -> UnityResult<Self> {
        let version = object.info.version;
        let tex_envs_size = r.read_i32()? as usize;
        let mut tex_envs = HashMap::with_capacity(tex_envs_size);
        for _ in 0..tex_envs_size {
            tex_envs.insert(r.read_aligned_string()?, UnityTexEnv::load(object, r)?);
        }
        let mut ints = HashMap::new();
        if version[0] >= 2021 {
            let ints_size = r.read_i32()? as usize;
            ints = HashMap::with_capacity(ints_size);
            for _ in 0..ints_size {
                ints.insert(r.read_aligned_string()?, r.read_i32()?);
            }
        }
        let floats_size = r.read_i32()? as usize;
        let mut floats = HashMap::with_capacity(tex_envs_size);
        for _ in 0..floats_size {
            floats.insert(r.read_aligned_string()?, r.read_f32()?);
        }
        let colors_size = r.read_i32()? as usize;
        let mut colors = HashMap::with_capacity(tex_envs_size);
        for _ in 0..colors_size {
            colors.insert(r.read_aligned_string()?, Color::from_array(r.read_f32_array::<4>()?));
        }
        Ok(Self { tex_envs, ints, floats, colors })
    }
}

pub struct UnityTexEnv<'a> {
    pub texture: PPtr<'a, Texture2D>,
    pub scale: Vector2,
    pub offset: Vector2,
}

impl<'a> UnityTexEnv<'a> {
    pub(super) fn load(object: &'a Object, r: &mut Reader) -> UnityResult<Self> {
        Ok(Self {
            texture: PPtr::load(object, r)?,
            scale: r.read_vector2()?,
            offset: r.read_vector2()?,
        })
    }
}
