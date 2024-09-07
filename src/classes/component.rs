use crate::classes::FromObject;
use crate::env::Object;
use crate::error::UnityResult;
use crate::reader::Reader;

use super::game_object::GameObject;
use super::pptr::PPtr;

pub struct Component<'a> {
    pub game_object: PPtr<'a, GameObject<'a>>,
}

impl<'a> FromObject<'a> for Component<'a> {
    fn load(object: &'a Object) -> UnityResult<Self> {
        let mut r = object.info.get_reader();
        Self::from_reader(object, &mut r)
    }

    fn class() -> super::ClassID {
        super::ClassID::Component
    }
}

impl<'a> Component<'a> {
    pub(crate) fn from_reader(object: &'a Object, r: &mut Reader) -> UnityResult<Self> {
        Ok(Self { game_object: PPtr::load(object, r)? })
    }
}
