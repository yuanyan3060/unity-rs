use crate::classes::FromObject;
use crate::env::Object;
use crate::error::UnityResult;

pub struct GameObject {}

impl<'a> FromObject<'a> for GameObject {
    fn load(_object: &Object) -> UnityResult<Self> {
        Err(crate::UnityError::Unimplemented)
    }
}
