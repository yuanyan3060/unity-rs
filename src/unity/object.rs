#![allow(unused)]
use std::sync::Arc;

use serde_json::json;

use super::{asset::SerializedType, class_id, type_node::TypeTreeNode, Reader, Result, AssetBundle};

#[derive(Clone)]
pub struct ObjectInfo {
    pub bundle: Arc<AssetBundle>,
    pub build_type: String,
    pub asset_version: u32,
    pub reader: Reader,
    pub bytes_start: i64,
    pub bytes_size: usize,
    pub type_id: i32,
    pub class_id: i32,
    pub is_destroyed: u16,
    pub stripped: u8,
    pub path_id: i64,
    pub serialized_type: SerializedType,
    pub version: [i32; 4],
}

