use crate::asset::SerializedType;
use crate::classes::ClassID;
use crate::error::UnityResult;
use crate::reader::{ByteOrder, Reader};
use crate::typetree::TypeTreeNode;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct ObjectInfo {
    pub build_type: String,
    pub asset_version: u32,
    pub bytes_start: usize,
    pub bytes_size: usize,
    pub data: Arc<Vec<u8>>,
    pub bytes_order: ByteOrder,
    pub type_id: i32,
    pub class_id: i32,
    pub is_destroyed: u16,
    pub stripped: u8,
    pub path_id: i64,
    pub serialized_type: SerializedType,
    pub version: [i32; 4],
}

impl ObjectInfo {
    pub fn get_reader(&self) -> Reader {
        Reader::new(&self.data[self.bytes_start..], self.bytes_order)
    }

    pub fn class(&self) -> ClassID {
        ClassID::from(self.class_id)
    }

    pub fn read_type_tree(&self) -> UnityResult<HashMap<String, Value>> {
        let mut r = self.get_reader();
        let mut result = HashMap::new();
        let nodes = &self.serialized_type.type_tree.nodes;
        let mut i = 1;
        while i < nodes.len() {
            let node = &nodes[i];
            let value = Self::read_type_tree_value(nodes, &mut r, &mut i)?;
            result.insert(node.name.clone(), value);
            i += 1;
        }
        Ok(result)
    }

    fn read_type_tree_value(nodes: &[TypeTreeNode], r: &mut Reader, index: &mut usize) -> UnityResult<Value> {
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
                let mut v = serde_json::Map::new();
                for _ in 0..size {
                    let key = Self::read_type_tree_value(&first, r, &mut 0)?;
                    let key = match key {
                        Value::String(_) => key.as_str().unwrap().to_string(),
                        _ => key.to_string(),
                    };
                    let value_ = Self::read_type_tree_value(&second, r, &mut 0)?;
                    v.insert(key.to_string(), value_);
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
                        *j += 1;
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
