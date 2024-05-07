use crate::classes::FromObject;
use crate::env::Object;
use crate::error::UnityResult;

pub struct MonoScript {

}

impl<'a> FromObject<'a> for MonoScript {
    fn load(_object: &Object) -> UnityResult<Self> {
        Err(crate::UnityError::Unimplemented)
    }
}
