#![allow(unused)]
use std::borrow::Cow;

use bytes::Bytes;

use crate::unity::{object::ObjectInfo, Result, FromObject, Object};

pub struct TextAsset {
    pub name: String,
    pub script: Bytes,
}

impl FromObject for TextAsset {
    fn load(object: &Object) -> Result<Self> {
        let mut r = object.info.reader.clone();
        r.set_offset(object.info.bytes_start as usize)?;
        let name = r.read_aligned_string()?;
        let length = r.read_i32()?;
        let script = r.read_u8_list(length as usize)?;
        Ok(Self { name, script })
    }
}
impl TextAsset {
    pub fn script_string(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.script)
    }
}
