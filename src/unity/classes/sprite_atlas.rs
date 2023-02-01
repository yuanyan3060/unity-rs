use bytes::Bytes;

use crate::unity::{object::ObjectInfo, Result, FromObject, Object};

pub struct SpriteAtlas {
    pub name: String,
}
impl FromObject for SpriteAtlas {
    fn load(object: &Object) -> Result<Self> {
        todo!()
    }
}

