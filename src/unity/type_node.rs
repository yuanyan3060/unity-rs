use bytes::Bytes;

#[derive(Default, Clone)]
pub struct TypeTreeNode {
    pub(super) type_: String,
    pub(super) name: String,
    pub(super) size: i32,
    pub(super) index: i32,
    pub(super) type_flag: i32,
    pub(super) version: i32,
    pub(super) meta_flag: i32,
    pub(super) level: i32,
    pub(super) type_str_offset: usize,
    pub(super) name_str_offset: usize,
    pub(super) ref_type_hash: u64,
}
#[derive(Default, Clone)]
pub struct TypeTree {
    pub(super) nodes: Vec<TypeTreeNode>,
    pub(super) string_buffer: Bytes,
}
