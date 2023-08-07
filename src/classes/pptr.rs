use crate::classes::FromObject;
use crate::env::{Env, Object};
use crate::error::UnityResult;
use crate::reader::Reader;
use std::{any::type_name, marker::PhantomData};

pub struct PPtr<'a, T: FromObject<'a> + 'a> {
    env: &'a Env,
    pub file_id: i32,
    pub path_id: i64,
    target: PhantomData<T>,
}

impl<'a, T: FromObject<'a>> PPtr<'a, T> {
    pub fn load(object: &'a Object, r: &mut Reader) -> UnityResult<Self> {
        let file_id = r.read_i32()?;
        let path_id = if object.info.asset_version < 14 { r.read_i32()? as i64 } else { r.read_i64()? };
        Ok(Self {
            env: object.env,
            file_id,
            path_id,
            target: PhantomData::default(),
        })
    }

    pub fn get_obj(&self) -> Option<Object<'a>> {
        if self.path_id == 0 {
            return None;
        }
        for bundle in &self.env.bundles {
            for asset in &bundle.assets {
                for info in &asset.objects_info {
                    if info.path_id == self.path_id {
                        let obj = Object {
                            env: self.env,
                            bundle,
                            asset,
                            info: info.clone(),
                            cache: self.env.cache.clone(),
                        };
                        return Some(obj);
                    }
                }
            }
        }
        return None;
    }
}

impl<'a, T: FromObject<'a>> std::fmt::Debug for PPtr<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = type_name::<T>();
        let (_, t) = t.rsplit_once("::").unwrap();
        write!(f, "PPtr {{ file_id: {}, path_id: {}, type: {} }}", self.file_id, self.path_id, t)
    }
}
