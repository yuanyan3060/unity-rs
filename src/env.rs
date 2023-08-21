use crate::asset::Asset;
use crate::bundle::AssetBundle;
use crate::classes::{ClassID, FromObject};
use crate::error::UnityResult;
use crate::object::ObjectInfo;
use dashmap::DashMap;
use image::RgbaImage;
use serde_json::Value;
use std::collections::HashMap;

use std::sync::Arc;

pub struct ObjectIter<'a> {
    env: &'a Env,
    bundle_index: usize,
    asset_index: usize,
    obj_index: usize,
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = Object<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let bundle = self.env.bundles.get(self.bundle_index)?;
        let Some(asset) = bundle.assets.get(self.asset_index) else {
            self.asset_index = 0;
            self.bundle_index += 1;
            return self.next();
        };
        let Some(info) = asset.objects_info.get(self.obj_index) else{
            self.obj_index = 0;
            self.asset_index +=1;
            return self.next();
        };
        self.obj_index += 1;
        Some(Object {
            env: self.env,
            bundle,
            asset,
            info: info.clone(),
            cache: self.env.cache.clone(),
        })
    }
}

pub struct Env {
    pub bundles: Vec<AssetBundle>,
    pub cache: Arc<DashMap<i64, RgbaImage>>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        Self {
            bundles: Vec::new(),
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn load_from_slice(&mut self, src: &[u8]) -> UnityResult<()> {
        let bundle = AssetBundle::from_slice(src)?;
        self.bundles.push(bundle);
        Ok(())
    }

    pub fn objects(&self) -> ObjectIter {
        ObjectIter {
            env: self,
            bundle_index: 0,
            asset_index: 0,
            obj_index: 0,
        }
    }

    pub fn find_object(&self, path_id: i64) -> Option<Object> {
        self.objects().find(|i| i.info.path_id == path_id)
    }
}

pub struct Object<'a> {
    pub env: &'a Env,
    pub bundle: &'a AssetBundle,
    pub asset: &'a Asset,
    pub info: ObjectInfo,
    pub cache: Arc<DashMap<i64, RgbaImage>>,
}

impl<'a> Object<'a> {
    pub fn read<T: FromObject<'a>>(&'a self) -> UnityResult<T> {
        T::load(self)
    }

    pub fn class(&self) -> ClassID {
        ClassID::from(self.info.class_id)
    }

    pub fn read_type_tree(&self) -> UnityResult<HashMap<String, Value>> {
        self.info.read_type_tree()
    }
}
