use crate::asset::Asset;
use crate::bundle::AssetBundle;
use crate::classes::{ClassID, FromObject};
use dashmap::DashMap;
use image::RgbaImage;
use serde_json::Value;
use std::collections::HashMap;
use std::iter::zip;
use std::sync::Arc;

use crate::error::UnityResult;
use crate::object::ObjectInfo;

pub struct Env {
    pub bundles: Vec<AssetBundle>,
    pub assets: Vec<Vec<Asset>>,
    pub cache: Arc<DashMap<i64, RgbaImage>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            bundles: Vec::new(),
            assets: Vec::new(),
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn load_from_slice(&mut self, src: &[u8]) -> UnityResult<()> {
        let bundle = AssetBundle::from_slice(src)?;
        self.assets.push(bundle.get_assets()?);
        self.bundles.push(bundle);
        Ok(())
    }

    pub fn objects(&self) -> Vec<Object> {
        let mut result = Vec::new();
        for (bundle, assets) in zip(&self.bundles, &self.assets) {
            for asset in assets {
                for info in &asset.objects_info {
                    result.push(Object {
                        env: self,
                        bundle,
                        asset,
                        info: info.clone(),
                        cache: self.cache.clone(),
                    })
                }
            }
        }
        result
    }

    pub fn find_object(&self, path_id: i64) -> Option<Object> {
        for (bundle, assets) in zip(&self.bundles, &self.assets) {
            for asset in assets {
                for info in &asset.objects_info {
                    if info.path_id == path_id {
                        return Some(Object {
                            env: self,
                            bundle,
                            asset,
                            info: info.clone(),
                            cache: self.cache.clone(),
                        });
                    }
                }
            }
        }
        None
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
