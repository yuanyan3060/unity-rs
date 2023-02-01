use std::sync::Arc;

use super::common::common_string;
use super::object::ObjectInfo;
use super::type_node::{TypeTree, TypeTreeNode};
use super::{ByteOrder, Reader, Result, AssetBundle};
#[derive(Default)]
pub struct SerializedFileHeader {
    metadata_size: usize,
    file_size: usize,
    version: u32,
    data_offset: usize,
    endianess: u8,
    reserved: [u8;3],
}

#[derive(Default, Clone)]
pub struct SerializedType {
    pub(super) class_id: i32,
    pub(super) is_stripped_type: bool,
    pub(super) script_type_index: Option<i16>,
    pub(super) type_tree: TypeTree,
    pub(super) script_id: [u8; 16],
    pub(super) old_type_hash: [u8; 16],
    pub(super) type_dependencies: Vec<i32>,
    pub(super) klass_name: String,
    pub(super) name_space: String,
    pub(super) asm_name: String,
}

#[derive(Default)]
pub struct LocalSerializedObjectIdentifier {
    local_serialized_file_index: i32,
    local_identifier_in_file: i64,
}

#[derive(Default)]
pub struct FileIdentifier {
    guid: [u8; 16],
    type_: i32,
    path_name: String,
    //file_name: String,
}

pub struct Asset {
    //full_name: String,
    //path: String,
    //file_name: String,
    version: [i32; 4],
    build_type: String,
    header: SerializedFileHeader,
    file_endianess: u8,
    unity_version: String,
    target_platform: i32,
    enable_type_tree: bool,
    types: Vec<SerializedType>,
    big_id_enabled: bool,
    pub objects_info: Vec<ObjectInfo>,
    script_types: Vec<LocalSerializedObjectIdentifier>,
    externals: Vec<FileIdentifier>,
    ref_types: Vec<SerializedType>,
    user_information: String,
}

impl Asset {
    pub(super) fn new(data: bytes::Bytes, bundle:Arc<AssetBundle>) -> Result<Self> {
        let mut r = Reader::new(data, ByteOrder::Big);
        let mut ret = Self {
            //full_name: String::default(),
            //path: String::default(),
            //file_name: String::default(),
            version: [0; 4],
            build_type: String::default(),
            header: SerializedFileHeader::default(),
            file_endianess: 0,
            unity_version: String::default(),
            target_platform: 0,
            enable_type_tree: false,
            types: Vec::new(),
            big_id_enabled: false,
            objects_info: Vec::new(),
            script_types: Vec::new(),
            externals: Vec::new(),
            ref_types: Vec::new(),
            user_information: String::default(),
        };
        ret.header.metadata_size = r.read_u32()? as usize;
        ret.header.file_size = r.read_u32()? as usize;
        ret.header.version = r.read_u32()?;
        ret.header.data_offset = r.read_u32()? as usize;
        if ret.header.version >= 9 {
            ret.header.endianess = r.read_u8()?;
            ret.header.reserved = r.read_u8_array()?;
            ret.file_endianess = ret.header.endianess;
        } else {
            r.set_offset(ret.header.file_size - ret.header.metadata_size)?;
            ret.file_endianess = r.read_u8()?;
        }
        if ret.header.version >= 22 {
            ret.header.metadata_size = r.read_u32()? as usize;
            ret.header.file_size = r.read_i64()? as usize;
            ret.header.data_offset = r.read_i64()? as usize;
            r.read_i64()?;
        }
        if ret.file_endianess == 0 {
            r.set_little_order()
        }

        if ret.header.version >= 7 {
            ret.unity_version = r.read_string_utill_null()?;
            let mut s = String::new();
            for i in ret.unity_version.chars() {
                if i.is_ascii_alphabetic() {
                    ret.build_type = i.to_string();
                    s.push('.');
                } else {
                    s.push(i)
                }
            }
            let s = s.split(".");
            for (i, j) in s.into_iter().enumerate() {
                if i >= 4 {
                    break;
                }
                ret.version[i] = j.parse().unwrap()
            }
        }

        if ret.header.version >= 8 {
            ret.target_platform = r.read_i32()?;
        }

        if ret.header.version >= 13 {
            ret.enable_type_tree = r.read_bool()?;
        }

        let type_count = r.read_i32()?;
        for _ in 0..type_count {
            let st = ret.read_serialized_type(&mut r, false)?;
            ret.types.push(st)
        }
        if ret.header.version >= 7 && ret.header.version < 14 {
            ret.big_id_enabled = r.read_i32()? != 0;
        }
        let object_count = r.read_i32()?;
        for _ in 0..object_count {
            let mut object_info = ObjectInfo {
                bundle:bundle.clone(),
                build_type:ret.build_type.clone(),
                reader: r.clone(),
                asset_version: ret.header.version,
                bytes_start: 0,
                bytes_size: 0,
                type_id: 0,
                class_id: 0,
                is_destroyed: 0,
                stripped: 0,
                path_id: 0,
                serialized_type: SerializedType::default(),
                version: [0; 4],
            };
            if ret.big_id_enabled {
                object_info.path_id = r.read_i64()?;
            } else if ret.header.version < 14 {
                object_info.path_id = r.read_i32()? as i64;
            } else {
                r.align(4)?;
                object_info.path_id = r.read_i64()?;
            }
            if ret.header.version >= 22 {
                object_info.bytes_start = r.read_i64()?;
            } else {
                object_info.bytes_start = r.read_u32()? as i64;
            }
            object_info.bytes_start += ret.header.data_offset as i64;
            object_info.version = ret.version;
            object_info.bytes_size = r.read_u32()? as usize;
            object_info.type_id = r.read_i32()?;
            if ret.header.version < 16 {
                object_info.class_id = r.read_u16()? as i32;
                for i in &ret.types {
                    if i.class_id == object_info.type_id {
                        object_info.serialized_type = (*i).clone();
                        break;
                    }
                }
            } else {
                let type_ = ret.types[object_info.type_id as usize].clone();
                object_info.class_id = type_.class_id;
                object_info.serialized_type = type_;
            }
            if ret.header.version < 11 {
                object_info.is_destroyed = r.read_u16()?;
            }
            if ret.header.version >= 11 && ret.header.version < 17 {
                let script_type_index = r.read_i16()?;
                object_info.serialized_type.script_type_index = Some(script_type_index);
            }
            if ret.header.version == 15 || ret.header.version == 16 {
                object_info.stripped = r.read_u8()?;
            }
            ret.objects_info.push(object_info);
        }
        if ret.header.version >= 11 {
            let script_count = r.read_i32()?;
            for _ in 0..script_count {
                let mut script_type = LocalSerializedObjectIdentifier::default();
                script_type.local_serialized_file_index = r.read_i32()?;
                if ret.header.version < 14 {
                    script_type.local_identifier_in_file = r.read_i32()? as i64;
                } else {
                    r.align(4)?;
                    script_type.local_identifier_in_file = r.read_i64()?;
                }
                ret.script_types.push(script_type)
            }
        }
        let externals_count = r.read_i32()?;
        for _ in 0..externals_count {
            let mut external = FileIdentifier::default();
            if ret.header.version >= 6 {
                r.read_string_utill_null()?;
            }
            if ret.header.version >= 5 {
                external.guid = r.read_u8_array()?;
                external.type_ = r.read_i32()?;
            }
            external.path_name = r.read_string_utill_null()?;
            ret.externals.push(external)
        }
        if ret.header.version >= 20 {
            let ref_type_count = r.read_i32()?;
            for _ in 0..ref_type_count {
                let st = ret.read_serialized_type(&mut r, true)?;
                ret.ref_types.push(st)
            }
        }
        if ret.header.version >= 5 {
            ret.user_information = r.read_string_utill_null()?;
        }
        Ok(ret)
    }

    pub fn read_serialized_type(
        &mut self,
        r: &mut Reader,
        is_ref_type: bool,
    ) -> Result<SerializedType> {
        let mut result = SerializedType::default();
        result.class_id = r.read_i32()?;
        if self.header.version >= 16 {
            result.is_stripped_type = r.read_bool()?;
        }

        if self.header.version >= 17 {
            result.script_type_index = Some(r.read_i16()?);
        }

        if self.header.version >= 13 {
            if is_ref_type && result.script_type_index.is_some() {
                result.script_id = r.read_u8_array()?;
            } else if (self.header.version < 16 && result.class_id < 0)
                || (self.header.version >= 16 && result.class_id == 114)
            {
                result.script_id = r.read_u8_array()?;
            }
            result.old_type_hash = r.read_u8_array()?;
        }
        if self.enable_type_tree {
            if self.header.version >= 12 || self.header.version == 10 {
                self.read_type_tree_blob(r, &mut result.type_tree)?
            } else {
                self.read_type_tree(r, &mut result.type_tree, 0)?;
            }
            if self.header.version >= 21 {
                if is_ref_type {
                    result.klass_name = r.read_string_utill_null()?;
                    result.name_space = r.read_string_utill_null()?;
                    result.asm_name = r.read_string_utill_null()?;
                } else {
                    result.type_dependencies = r.read_i32_list_without_size()?;
                }
            }
        }
        Ok(result)
    }

    pub fn read_type_tree_blob(&mut self, r: &mut Reader, type_tree: &mut TypeTree) -> Result<()> {
        fn read_string(r: &mut Reader, offset: usize) -> Result<String> {
            let is_offset = offset & 0x80000000 == 0;
            if is_offset {
                r.set_offset(offset)?;
                return Ok(r.read_string_utill_null()?);
            }
            let offset = offset & 0x7FFFFFFF;
            match common_string(offset) {
                Some(s) => Ok(s.to_string()),
                None => Ok(offset.to_string()),
            }
        }

        let node_number = r.read_i32()?;
        let string_buffer_size = r.read_i32()? as usize;
        for _ in 0..node_number {
            let mut type_tree_node = TypeTreeNode::default();
            type_tree_node.version = r.read_u16()? as i32;
            type_tree_node.level = r.read_u8()? as i32;
            type_tree_node.type_flag = r.read_u8()? as i32;
            type_tree_node.type_str_offset = r.read_u32()? as usize;
            type_tree_node.name_str_offset = r.read_u32()? as usize;
            type_tree_node.size = r.read_i32()?;
            type_tree_node.index = r.read_i32()?;
            type_tree_node.meta_flag = r.read_i32()?;
            if self.header.version >= 19 {
                type_tree_node.ref_type_hash = r.read_u64()?;
            }
            type_tree.nodes.push(type_tree_node)
        }
        type_tree.string_buffer = r.read_u8_list(string_buffer_size)?;
        let mut string_buffer_reader = Reader::new(type_tree.string_buffer.clone(), ByteOrder::Big);
        for node in &mut type_tree.nodes {
            node.type_ = read_string(&mut string_buffer_reader, node.type_str_offset)?;
            node.name = read_string(&mut string_buffer_reader, node.name_str_offset)?;
        }
        Ok(())
    }

    pub fn read_type_tree(
        &mut self,
        r: &mut Reader,
        type_tree: &mut TypeTree,
        level: i32,
    ) -> Result<()> {
        let mut type_tree_node = TypeTreeNode::default();
        type_tree_node.level = level;
        type_tree_node.type_ = r.read_string_utill_null()?;
        type_tree_node.name = r.read_string_utill_null()?;
        type_tree_node.size = r.read_i32()?;
        if self.header.version == 2 {
            let _variable_count = r.read_i32()?;
        }
        if self.header.version != 3 {
            type_tree_node.index = r.read_i32()?;
        }
        type_tree_node.type_flag = r.read_i32()?;
        type_tree_node.version = r.read_i32()?;
        if self.header.version != 3 {
            type_tree_node.meta_flag = r.read_i32()?;
        }
        type_tree.nodes.push(type_tree_node);
        for _ in 0..r.read_i32()? {
            self.read_type_tree(r, type_tree, level + 1)?;
        }
        Ok(())
    }
}
