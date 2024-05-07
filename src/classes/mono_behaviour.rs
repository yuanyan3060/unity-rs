use crate::classes::FromObject;
use crate::env::Object;
use crate::error::UnityResult;

use super::component::Component;
use super::game_object::GameObject;
use super::mono_script::MonoScript;
use super::pptr::PPtr;

pub struct MonoBehaviour<'a> {
    pub game_object: PPtr<'a, GameObject>,
    pub enable: bool,
    pub script: PPtr<'a, MonoScript>,
    pub name: String
}

impl<'a> FromObject<'a> for MonoBehaviour<'a> {
    fn load(object: &'a Object) -> UnityResult<Self> {
        let mut r = object.info.get_reader();
        let game_object = Component::from_reader(object, &mut r)?.game_object;
        let enable = r.read_bool()?;
        r.align(4)?;
        let script = PPtr::load(object, &mut r)?;
        let name = r.read_aligned_string()?;
        Ok(Self { game_object, enable, script, name })
    }
}
