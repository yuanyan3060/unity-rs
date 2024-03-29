use crate::error::UnityResult;
use crate::math::Vector3;
use crate::reader::Reader;

#[derive(Default, Debug)]
pub struct AnimationClip {
    pub center: Vector3,
    pub extent: Vector3,
}

impl AnimationClip {
    pub(super) fn load(r: &mut Reader) -> UnityResult<Self> {
        let center = r.read_vector3()?;
        let extent = r.read_vector3()?;
        Ok(Self { center, extent })
    }
}
