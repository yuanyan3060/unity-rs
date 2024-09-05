use crate::classes::FromObject;
use crate::env::Object;
use crate::error::UnityResult;

use super::pptr::PPtr;
use super::Component;

pub struct GameObject<'a> {
    pub components: Vec<PPtr<'a, Component<'a>>>,
    pub name: String,
}

impl<'a> FromObject<'a> for GameObject<'a> {
    fn load(object: &'a Object) -> UnityResult<Self> {
        let mut r = object.info.get_reader();
        let version = object.info.version;
        let count = r.read_i32()? as usize;
        let mut components = Vec::new();
        for _ in 0..count {
            if (version[0] == 5 && version[1] < 5) || version[0] < 5 {
                r.read_i32()?;
            }
            components.push(PPtr::load(object, &mut r)?);
        }
        let _layer = r.read_i32()?;
        let name = r.read_aligned_string()?;
        Ok(Self { components, name })
    }
}
