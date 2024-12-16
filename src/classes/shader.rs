use crate::{Object, UnityError, UnityResult};

use super::FromObject;

pub struct Shader {}

impl<'a> FromObject<'a> for Shader {
    fn load(_object: &Object) -> UnityResult<Self> {
        Err(UnityError::Unimplemented)
    }

    fn class() -> super::ClassID {
        super::ClassID::Shader
    }
}
