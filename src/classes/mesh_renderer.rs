use crate::UnityResult;

use super::{pptr::PPtr, renderer::SubMeshInfo, FromObject, GameObject, Material, Renderer};

pub struct MeshRenderer<'a> {
    pub game_object: PPtr<'a, GameObject<'a>>,
    pub materials: Vec<PPtr<'a, Material<'a>>>,
    pub sub_mesh_info: Option<SubMeshInfo>,
}

impl<'a> FromObject<'a> for MeshRenderer<'a> {
    fn load(object: &'a crate::Object<'a>) -> UnityResult<Self> {
        let Renderer { game_object, materials, sub_mesh_info } = Renderer::load(object)?;
        Ok(Self { game_object, materials, sub_mesh_info })
    }

    fn class() -> super::ClassID {
        super::ClassID::MeshRenderer
    }
}
