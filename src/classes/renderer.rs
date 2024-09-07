use either::Either;

use crate::{object::ObjectInfo, reader::Reader, UnityResult};

use super::{pptr::PPtr, FromObject, GameObject, Material, Transform};

pub struct StaticBatchInfo {
    pub first_sub_mesh: u16,
    pub sub_mesh_count: u16,
}

impl StaticBatchInfo {
    pub(super) fn load(_object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        Ok(Self {
            first_sub_mesh: r.read_u16()?,
            sub_mesh_count: r.read_u16()?,
        })
    }
}

pub enum SubMeshInfo {
    StaticBatchInfo(StaticBatchInfo),
    SubsetIndices(Vec<u32>),
}

impl SubMeshInfo {
    pub fn sub_mesh_indices<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        match self {
            SubMeshInfo::StaticBatchInfo(static_batch_info) => {
                let left = static_batch_info.first_sub_mesh..(static_batch_info.first_sub_mesh + static_batch_info.sub_mesh_count);
                let left = left.map(|x| x as u32);
                Either::Left(left)
            },
            SubMeshInfo::SubsetIndices(subset_indices) => {
                Either::Right(subset_indices.iter().cloned())
            },
        }
    }
}

pub struct Renderer<'a> {
    pub game_object: PPtr<'a, GameObject<'a>>,
    pub materials: Vec<PPtr<'a, Material<'a>>>,
    pub sub_mesh_info: Option<SubMeshInfo>,
}

impl<'a> FromObject<'a> for Renderer<'a> {
    fn load(object: &'a crate::Object<'a>) -> UnityResult<Self> {
        let version = object.info.version;
        let mut r = object.info.get_reader();
        let game_object = PPtr::load(object, &mut r)?;
        if version[0] < 5 {
            let _enabled = r.read_bool()?;
            let _cast_shadows = r.read_bool()?;
            let _receive_shadows = r.read_bool()?;
            let _lightmap_index = r.read_u8()?;
        } else {
            if version[0] > 5 || (version[0] == 5 && version[1] >= 4) {
                let _enabled = r.read_bool()?;
                let _cast_shadows = r.read_u8()?;
                let _receive_shadows = r.read_u8()?;
                if version[0] > 2017 || (version[0] == 2017 && version[1] >= 2) {
                    let _dynamic_occludee = r.read_u8()?;
                }
                if version[0] >= 2021 {
                    let _static_shadow_caster = r.read_u8()?;
                }
                let _motion_vectors = r.read_u8()?;
                let _light_probe_usage = r.read_u8()?;
                let _reflection_probe_usage = r.read_u8()?;
                if version[0] > 2019 || (version[0] == 2019 && version[1] >= 3) {
                    let _ray_tracing_mode = r.read_u8()?;
                }
                if version[0] >= 2020 {
                    let _ray_trace_procedural = r.read_u8()?;
                }
                r.align(4)?;
            } else {
                let _enabled = r.read_bool()?;
                r.align(4)?;
                let _cast_shadows = r.read_u8()?;
                let _receive_shadows = r.read_bool()?;
                r.align(4)?;
            }
            if version[0] >= 2018 {
                let _rendering_layer_mask = r.read_u32()?;
            }
            if version[0] > 2018 || (version[0] == 2018 && version[1] >= 3) {
                let _renderer_priority = r.read_i32()?;
            }
            let _lightmap_index = r.read_u16()?;
            let _lightmap_index_dynamic = r.read_u16()?;
        }
        if version[0] >= 3 {
            let _lightmap_tiling_offset = r.read_vector4()?;
        }
        if version[0] >= 5 {
            let _lightmap_tiling_offset_dynamic = r.read_vector4()?;
        }
        let materials_size = r.read_i32()?;
        let mut materials = Vec::with_capacity(materials_size as usize);
        for _ in 0..materials_size {
            materials.push(PPtr::load(object, &mut r)?);
        }
        let mut sub_mesh_info = None;
        if version[0] < 3 {
            let _lightmap_tiling_offset = r.read_vector4()?;
        } else {
            if version[0] > 5 || (version[0] == 5 && version[1] >= 5) {
                sub_mesh_info = Some(SubMeshInfo::StaticBatchInfo(StaticBatchInfo::load(&object.info, &mut r)?))
            } else {
                let size = r.read_i32()? as usize;
                sub_mesh_info = Some(SubMeshInfo::SubsetIndices(r.read_u32_list(size)?))
            }
            let _static_batch_root = PPtr::<Transform>::load(object, &mut r)?;
        }
        if version[0] > 5 || (version[0] == 5 && version[1] >= 4) {
            let _probe_anchor = PPtr::<Transform>::load(object, &mut r)?;
            let _light_probe_volume_override = PPtr::<GameObject>::load(object, &mut r)?;
        } else if version[0] > 3 || (version[0] == 3 && version[1] >= 5) {
            let _use_light_probes = r.read_bool()?;
            r.align(4)?;
            if version[0] >= 5 {
                let _reflection_probe_usage = r.read_i32()?;
            }
            let _light_probe_anchor = PPtr::<Transform>::load(object, &mut r)?;
        }

        if version[0] > 4 || (version[0] == 4 && version[1] >= 3) {
            if version[0] == 4 && version[1] == 3 {
                let _sorting_layer = r.read_i16()?;
            } else {
                let _sorting_layer_id = r.read_u32()?;
            }
            let _sorting_order = r.read_i16()?;
            r.align(4)?;
        }
        Ok(Self { game_object, materials, sub_mesh_info })
    }

    fn class() -> super::ClassID {
        super::ClassID::Renderer
    }
}
