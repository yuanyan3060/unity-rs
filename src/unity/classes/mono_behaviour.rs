#![allow(unused)]
use std::borrow::Cow;

use crate::unity::{object::ObjectInfo, Result, FromObject, Object};

pub struct MonoBehaviour {
    pub name: String,
}

impl FromObject for MonoBehaviour {
    fn load(object: &Object) -> Result<Self> {
        let mut r = object.info.reader.clone();
        r.set_offset(object.info.bytes_start as usize)?;
        let name = r.read_aligned_string()?;
        Ok(Self { name})
    }
}

