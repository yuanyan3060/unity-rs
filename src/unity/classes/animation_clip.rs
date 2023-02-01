use bytes::Bytes;

use crate::unity::{object::ObjectInfo, FromObject, Object, Reader, Result, math::Vector3};


#[derive(Default, Debug)]
pub struct AABB {
    pub center: Vector3,
    pub extent: Vector3,
}

impl AABB {
     pub(super) fn load(object: &Object, r: &mut Reader) -> Result<Self> {
        let center = r.read_vector3()?;
        let extent = r.read_vector3()?;
        Ok(Self { center, extent })
    }
}
