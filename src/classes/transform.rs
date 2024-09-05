use crate::classes::FromObject;
use crate::env::Object;
use crate::error::UnityResult;
use crate::math::{Quaternion, Vector3};

use super::game_object::GameObject;
use super::pptr::PPtr;

pub struct Transform<'a> {
    pub game_object: PPtr<'a, GameObject<'a>>,
    pub local_rotation: Quaternion,
    pub local_position: Vector3,
    pub local_scale: Vector3,
    pub children: Vec<PPtr<'a, Self>>,
    pub father: PPtr<'a, Self>,
}

impl<'a> FromObject<'a> for Transform<'a> {
    fn load(object: &'a Object) -> UnityResult<Self> {
        let mut r = object.info.get_reader();
        Ok(Self {
            game_object: PPtr::<GameObject>::load(object, &mut r)?,
            local_rotation: Quaternion::from_array(r.read_f32_array::<4>()?),
            local_position: r.read_vector3()?,
            local_scale: r.read_vector3()?,
            children: {
                let count = r.read_i32()? as usize;
                let mut children = Vec::with_capacity(count);
                for _ in 0..count {
                    children.push(PPtr::load(object, &mut r)?);
                }
                children
            },
            father: PPtr::<Self>::load(object, &mut r)?,
        })
    }
}
