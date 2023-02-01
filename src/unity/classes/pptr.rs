#![allow(unused)]
use std::{borrow::Cow, collections::HashMap, marker::PhantomData, sync::Arc, any::type_name};
use std::fmt;
use bytes::Bytes;
use serde::Serializer;

use crate::unity::{
    bundle::AssetBundle, object::ObjectInfo, Env, Error, FromObject, Object, Reader, Result,
};

pub struct PPtr<T: FromObject> {
    pub object_info_map: Arc<HashMap<i64, ObjectInfo>>,
    pub file_id: i32,
    pub path_id: i64,
    target: PhantomData<T>,
}

impl<T: FromObject> std::default::Default for PPtr<T> {
    fn default() -> Self {
        Self {
            object_info_map: Default::default(),
            file_id: Default::default(),
            path_id: Default::default(),
            target: Default::default(),
        }
    }
}

impl<T: FromObject> PPtr<T> {
    pub fn load(object: &Object, r: &mut Reader) -> Result<Self> {
        let object_info_map = object.info_map.clone();
        let bundle = object.info.bundle.clone();
        let file_id = r.read_i32()?;
        let path_id = if object.info.asset_version < 14 {
            r.read_i32()? as i64
        } else {
            r.read_i64()?
        };
        Ok(Self {
            object_info_map,
            file_id,
            path_id,
            target: PhantomData::default(),
        })
    }

    pub fn get_obj(&self) -> Result<T> {
        for (k, v) in &*self.object_info_map {
            if v.path_id != self.path_id {
                continue;
            }
            return Ok(Object {
                info: v.clone(),
                info_map: self.object_info_map.clone(),
            }
            .read()?);
        }
        return Err(Error::InvalidValue);
    }
}

impl<T: FromObject> std::fmt::Debug for PPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = type_name::<T>();
        let t = t.strip_prefix("arknights_unpack_rs::unity::classes::").expect(t);
        let (_, t) = t.split_once("::").unwrap();
        if f.is_human_readable(){
            write!(f, "PPtr {{ 
    file_id: {}, 
    path_id: {}, 
    type: {} 
}}", self.file_id, self.path_id, t)
        }else{
            write!(f, "PPtr {{ file_id: {}, path_id: {}, type: {} }}", self.file_id, self.path_id, t)
        }
    }
}
