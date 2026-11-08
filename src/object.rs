use crate::asset::{BuildType, SerializedType};
use crate::classes::ClassID;
use crate::reader::{ByteOrder, Eof, Reader};
use crate::typetree::TypeTreeNode;
use serde::de::DeserializeOwned;
use std::fmt::Display;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ObjectInfo {
    pub build_type: BuildType,
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
    pub fn get_reader(&'_ self) -> Reader<'_> {
        Reader::new(&self.data[self.bytes_start..], self.bytes_order)
    }

    pub fn class(&self) -> ClassID {
        ClassID::from(self.class_id)
    }

    pub fn read_type_tree<T: DeserializeOwned>(&self) -> Result<T, ReadTypeTreeError> {
        let mut reader = self.get_reader();
        let nodes = &self.serialized_type.type_tree.nodes;
        let mut de = Deserializer { nodes, index: 0, reader: &mut reader };

        let result = T::deserialize(&mut de)?;
        Ok(result)
    }
}

#[derive(Debug)]
pub enum ReadTypeTreeError {
    BufEof,
    NodeEof,
    Custom(String),
}

impl Display for ReadTypeTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadTypeTreeError::BufEof => write!(f, "BufEof"),
            ReadTypeTreeError::NodeEof => write!(f, "NodeEof"),
            ReadTypeTreeError::Custom(custom) => write!(f, "Custom({})", custom),
        }
    }
}

impl From<Eof> for ReadTypeTreeError {
    fn from(_: Eof) -> Self {
        Self::BufEof
    }
}

impl serde::de::StdError for ReadTypeTreeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl serde::de::Error for ReadTypeTreeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

pub struct Deserializer<'a> {
    nodes: &'a [TypeTreeNode],
    index: usize,
    reader: &'a mut Reader<'a>,
}

impl<'a> Deserializer<'a> {
    pub fn new(nodes: &'a [TypeTreeNode], reader: &'a mut Reader<'a>) -> Self {
        Self { nodes, index: 0, reader }
    }
}

impl<'de, 'a: 'de> serde::Deserializer<'de> for &mut Deserializer<'a> {
    type Error = ReadTypeTreeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some(node) = self.nodes.get(self.index) else {
            return Err(ReadTypeTreeError::NodeEof);
        };
        let mut align = (node.meta_flag & 0x4000) != 0;
        let val = match node.type_.as_str() {
            "SInt8" => visitor.visit_i8(self.reader.read_i8()?),
            "UInt8" | "char" => visitor.visit_u8(self.reader.read_u8()?),
            "short" | "SInt16" => visitor.visit_i16(self.reader.read_i16()?),
            "UInt16" | "unsigned short" => visitor.visit_u16(self.reader.read_u16()?),
            "int" | "SInt32" => visitor.visit_i32(self.reader.read_i32()?),
            "UInt32" | "unsigned int" | "Type*" => visitor.visit_u32(self.reader.read_u32()?),
            "long long" | "SInt64" => visitor.visit_i64(self.reader.read_i64()?),
            "UInt64" | "unsigned long long" | "FileSize" => visitor.visit_u64(self.reader.read_u64()?),
            "float" => visitor.visit_f32(self.reader.read_f32()?),
            "double" => visitor.visit_f64(self.reader.read_f64()?),
            "bool" => visitor.visit_bool(self.reader.read_bool()?),
            "string" => {
                self.index += 3;
                visitor.visit_string(self.reader.read_aligned_string()?)
            }
            "TypelessData" => {
                let size = self.reader.read_i32()?;
                let v = self.reader.read_u8_list(size as usize)?;
                self.index += 2;
                visitor.visit_byte_buf(v)
            }
            "map" => {
                if let Some(next_node) = self.nodes.get(self.index + 1) {
                    if next_node.meta_flag & 0x4000 != 0 {
                        align = true;
                    }
                }

                let map = get_level_length(self.nodes, self.index);

                let first = self.index + 4;
                let second = get_level_length(self.nodes, self.index + 4) + first;

                self.index += map - 1;
                let size = self.reader.read_i32()? as usize;
                visitor.visit_map(MapAccess { de: self, first, second, index: 0, size })
            }
            _ => {
                let next_node = self.nodes.get(self.index + 1);
                let array_node = match next_node {
                    Some(next_node) if next_node.type_ == "Array" => Some(next_node),
                    _ => None,
                };

                match array_node {
                    Some(array_node) => {
                        if array_node.meta_flag & 0x4000 != 0 {
                            align = true;
                        }
                        let vector = get_level_length(self.nodes, self.index);
                        let offset = self.index + 3;
                        let end_offset = self.index + vector - 1;
                        let size = self.reader.read_i32()? as usize;
                        visitor.visit_seq(SeqAccess { de: self, offset, index: 0, size, end_offset })
                    }
                    None => {
                        let vector = get_level_length(self.nodes, self.index);
                        let end = self.index + vector - 1;
                        self.index += 1;
                        visitor.visit_map(StructAccess { de: self, end, finish: false })
                    }
                }
            }
        };

        if align {
            self.reader.align(4)?;
        }

        val
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct MapAccess<'a, 'b: 'a> {
    de: &'a mut Deserializer<'b>,
    first: usize,
    second: usize,
    index: usize,
    size: usize,
}

impl<'de, 'a, 'b: 'a + 'de> serde::de::MapAccess<'de> for MapAccess<'a, 'b> {
    type Error = ReadTypeTreeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.index >= self.size {
            return Ok(None);
        }
        let index = self.de.index;
        self.de.index = self.first;
        let val = seed.deserialize(&mut *self.de);
        self.de.index = index;
        Ok(Some(val?))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let index = self.de.index;
        self.de.index = self.second;
        let val = seed.deserialize(&mut *self.de);
        self.de.index = index;
        self.index += 1;
        val
    }
}

struct SeqAccess<'a, 'b: 'a> {
    de: &'a mut Deserializer<'b>,
    offset: usize,
    index: usize,
    size: usize,
    end_offset: usize,
}

impl<'de, 'a, 'b: 'a + 'de> serde::de::SeqAccess<'de> for SeqAccess<'a, 'b> {
    type Error = ReadTypeTreeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.index >= self.size {
            self.de.index = self.end_offset;
            return Ok(None);
        }

        let offset = self.de.index;
        self.de.index = self.offset;
        let val = seed.deserialize(&mut *self.de);
        self.de.index = offset;
        self.index += 1;

        Ok(Some(val?))
    }
}

struct StructAccess<'a, 'b: 'a> {
    de: &'a mut Deserializer<'b>,
    end: usize,
    finish: bool,
}

impl<'a, 'b: 'a> StructAccess<'a, 'b> {
    fn check_finish(&self) -> bool {
        if self.de.index >= self.de.nodes.len() {
            return true;
        }

        if self.de.index >= self.end {
            return true;
        }

        false
    }
}

impl<'de, 'a, 'b: 'a + 'de> serde::de::MapAccess<'de> for StructAccess<'a, 'b> {
    type Error = ReadTypeTreeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.finish {
            return Ok(None);
        }
        let Some(node) = self.de.nodes.get(self.de.index) else {
            return Ok(None);
        };
        Ok(Some(seed.deserialize(Field { key: &node.name })?))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        if self.check_finish() {
            self.finish = true;
        } else {
            self.de.index += 1;
        }
        Ok(val)
    }
}

struct Field<'de> {
    key: &'de str,
}

impl<'de> serde::de::Deserializer<'de> for Field<'de> {
    type Error = ReadTypeTreeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_str(self.key)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

fn get_level_length(nodes: &[TypeTreeNode], idx: usize) -> usize {
    let Some(nodes) = nodes.get(idx..) else {
        return 0;
    };
    match nodes.split_first() {
        Some((first, others)) => others.iter().take_while(|x| x.level > first.level).count() + 1,
        None => 0,
    }
}
