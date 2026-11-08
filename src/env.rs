use crate::asset::SerializedFile;
use crate::bundle::{BundleFileLoader, FileLoader};
use crate::classes::{ClassID, FromObject};
use crate::error::UnityResult;
use crate::object::{ObjectInfo, ReadTypeTreeError};
use dashmap::DashMap;
use image::RgbaImage;
use serde::de::DeserializeOwned;

use std::fmt::Debug;
use std::sync::Arc;

pub struct ObjectIter<'a> {
    env: &'a Env,
    asset_index: usize,
    obj_index: usize,
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = Object<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let asset = self.env.serialized_files.get(self.asset_index)?;
        let Some(info) = asset.objects_info.get(self.obj_index) else {
            self.obj_index = 0;
            self.asset_index += 1;
            return self.next();
        };
        self.obj_index += 1;
        Some(Object {
            env: self.env,
            asset,
            info,
            cache: self.env.cache.clone(),
        })
    }
}

pub struct Env<F: FileLoader = BundleFileLoader> {
    pub file_loader: F,
    pub metadata: Vec<F::MetaData>,
    pub serialized_files: Vec<SerializedFile>,
    pub cache: Arc<DashMap<i64, RgbaImage>>,
    pub loaded_files: Arc<DashMap<String, Arc<Vec<u8>>>>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Env")
    }
}

impl Env {
    pub fn new() -> Self {
        Self {
            file_loader: BundleFileLoader,
            metadata: Vec::new(),
            serialized_files: Vec::new(),
            cache: Arc::new(DashMap::new()),
            loaded_files: Arc::new(DashMap::new()),
        }
    }

    pub fn with_loader<F: FileLoader>(self, file_loader: F) -> Env<F> {
        Env {
            file_loader,
            metadata: Vec::new(),
            serialized_files: self.serialized_files,
            cache: self.cache,
            loaded_files: self.loaded_files,
        }
    }

    pub fn load_from_slice(&mut self, src: &[u8]) -> UnityResult<()> {
        let output = self.file_loader.load(src)?;
        self.serialized_files.extend(output.serialized_files);
        for loaded_file in output.loaded_files {
            self.loaded_files.insert(loaded_file.name, loaded_file.data);
        }

        Ok(())
    }

    pub fn objects(&self) -> ObjectIter<'_> {
        ObjectIter { env: self, asset_index: 0, obj_index: 0 }
    }

    pub fn get_loaded_file(&self, name: &str) -> Option<Arc<Vec<u8>>> {
        self.loaded_files.get(name).map(|x| x.value().clone())
    }

    pub fn find_object(&self, path_id: i64) -> Option<Object<'_>> {
        self.objects().find(|i| i.info.path_id == path_id)
    }

    pub fn find_object_with_class<'a, T: FromObject<'a>>(&'a self, path_id: i64) -> Option<Object<'a>> {
        self.objects().find(|i| i.info.path_id == path_id && i.info.class() == T::class())
    }
}

#[derive(Debug)]
pub struct Object<'a> {
    pub env: &'a Env,
    pub asset: &'a SerializedFile,
    pub info: &'a ObjectInfo,
    pub cache: Arc<DashMap<i64, RgbaImage>>,
}

impl<'a> Object<'a> {
    pub fn read<T: FromObject<'a>>(&'a self) -> UnityResult<T> {
        T::load(self)
    }

    pub fn class(&self) -> ClassID {
        ClassID::from(self.info.class_id)
    }

    pub fn read_type_tree<T: DeserializeOwned>(&self) -> Result<T, ReadTypeTreeError> {
        self.info.read_type_tree()
    }
}
