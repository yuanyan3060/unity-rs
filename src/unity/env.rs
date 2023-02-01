use std::{sync::Arc, collections::HashMap};

use bytes::Bytes;
use serde_json::json;

use super::{AssetBundle, Asset, Result, bundle::FileType, object::ObjectInfo, class_id, type_node::TypeTreeNode, Reader};

pub struct Env{
    pub(super) bundle:Arc<AssetBundle>,
    pub(super) assets:Vec<Asset>,
    pub object_info_map: Arc<HashMap<i64, ObjectInfo>>
}


impl Env {
    pub fn load(data:Bytes)->Result<Self>{
        let bundle = Arc::new(AssetBundle::load(data)?);
        let mut assets = Vec::new();
        for f in &bundle.files {
            if let FileType::AssetsFile = AssetBundle::check_file_type(f.clone())? {
                let a = Asset::new(f.clone(), bundle.clone())?;
                assets.push(a);
            }
        }
        let mut object_info_map = HashMap::new();
        for a in &assets{
            for i in &a.objects_info{
                object_info_map.insert(i.path_id, i.clone());
            }
        }
        Ok(Env { bundle, assets, object_info_map:Arc::new(object_info_map)})
    }

    pub fn objects(&self)->HashMap<i64, Object>{
        let mut result = HashMap::new();
        for (k, v) in &*self.object_info_map{
            result.insert(*k, Object{info:v.clone(), info_map:self.object_info_map.clone()});
        }
        result
    }
}

pub struct Object{
    pub info: ObjectInfo,
    pub info_map: Arc<HashMap<i64, ObjectInfo>>
}

pub trait FromObject {
    fn load(object: &Object) -> Result<Self>
    where
        Self: Sized;
}

impl Object {
    pub fn read<T: FromObject>(&self) -> Result<T> {
        T::load(self)
    }

    pub fn class(&self) -> class_id::ClassIDType {
        class_id::ClassIDType::from(self.info.class_id)
    }

    pub fn read_type_tree(&self) -> Result<serde_json::Map<String, serde_json::Value>> {
        let mut r = self.info.reader.clone();
        r.set_offset(self.info.bytes_start as usize)?;
        let mut result = serde_json::Map::new();
        let nodes = &self.info.serialized_type.type_tree.nodes;
        let mut i = 1;
        loop {
            if i >= nodes.len() {
                break;
            }
            let node = &nodes[i];
            let value = Self::read_type_tree_value(&nodes, &mut r, &mut i)?;
            result.insert(node.name.clone(), value);
            
            i += 1;
        }
        Ok(result)
    }

    fn read_type_tree_value(
        nodes: &[TypeTreeNode],
        r: &mut Reader,
        index: &mut usize,
    ) -> Result<serde_json::Value> {
        fn get_nodes(nodes: &[TypeTreeNode], index: usize) -> Vec<TypeTreeNode> {
            let mut result = vec![nodes[index].clone()];
            let level = nodes[index].level;
            for node in &nodes[index + 1..nodes.len()] {
                if node.level <= level {
                    return result;
                }
                result.push(node.clone())
            }
            result
        }
        let node = &nodes[*index];
        let mut align = (node.meta_flag & 0x4000) != 0;
        let value = match node.type_.as_str() {
            "SInt8" => json!(r.read_i8()?),
            "UInt8" | "char" => json!(r.read_u8()?),
            "short" | "SInt16" => json!(r.read_i16()?),
            "UInt16" | "unsigned short" => json!(r.read_u16()?),
            "int" | "SInt32" => json!(r.read_i32()?),
            "UInt32" | "unsigned int" | "Type*" => json!(r.read_u32()?),
            "long long" | "SInt64" => json!(r.read_i64()?),
            "UInt64" | "unsigned long long" | "FileSize" => json!(r.read_u64()?),
            "float" => json!(r.read_f32()?),
            "double" => json!(r.read_f64()?),
            "bool" => json!(r.read_bool()?),
            "string" => {
                let v = json!(r.read_aligned_string()?);
                *index += 3;
                v
            }
            "map" => {
                if nodes[*index + 1].meta_flag & 0x4000 != 0 {
                    align = true;
                }
                let map_ = get_nodes(nodes, *index);
                *index += map_.len() - 1;
                let first = get_nodes(&map_, 4);
                let second = get_nodes(&map_, 4 + first.len());
                let size = r.read_i32()? as usize;
                /*let mut v = Vec::with_capacity(size);
                for _ in 0..size {
                    let key = Self::read_type_tree_value(&first, r, &mut 0)?;
                    let value_ = Self::read_type_tree_value(&second, r, &mut 0)?;
                    v.push(json!({key.as_str().unwrap():value_}))
                }*/
                let mut v = serde_json::Map::new();
                for _ in 0..size {
                    let key = Self::read_type_tree_value(&first, r, &mut 0)?;
                    let key = match key {
                        serde_json::Value::String(_) => key.as_str().unwrap().to_string(),
                        _=>key.to_string()
                    };
                    let value_ = Self::read_type_tree_value(&second, r, &mut 0)?;
                    v.insert(key.to_string(),value_);
                }
                json!(v)
            }
            "TypelessData" => {
                let size = r.read_i32()?;
                let v = r.read_u8_list(size as usize)?;
                *index += 2;
                json!(v.to_vec())
            }
            _ => {
                if *index < nodes.len() - 1 && nodes[*index + 1].type_ == "Array" {
                    if (nodes[*index + 1].meta_flag & 0x4000) != 0 {
                        align = true;
                    }
                    let vector = get_nodes(nodes, *index);
                    *index += vector.len() - 1;
                    let size = r.read_i32()? as usize;
                    let mut v = Vec::new();
                    for _ in 0..size {
                        v.push(Self::read_type_tree_value(&vector, r, &mut 3)?)
                    }
                    json!(v)
                } else {
                    let clz = get_nodes(nodes, *index);
                    *index += clz.len() - 1;
                    let mut v = serde_json::Map::new();
                    let j = &mut 1;
                    loop {
                        if *j >= clz.len() {
                            break;
                        }
                        let clz_node = &clz[*j];
                        v.insert(clz_node.name.clone(), Self::read_type_tree_value(&clz, r, j)?);
                        *j+=1;
                    }
                    json!(v)
                }
            }
        };
        if align {
            r.align(4)?;
        }
        Ok(value)
    }
}