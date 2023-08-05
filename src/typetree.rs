#[derive(Default, Clone)]
pub struct TypeTreeNode {
    pub type_: String,
    pub name: String,
    pub size: i32,
    pub index: i32,
    pub type_flag: i32,
    pub version: i32,
    pub meta_flag: i32,
    pub level: i32,
    pub type_str_offset: usize,
    pub name_str_offset: usize,
    pub ref_type_hash: u64,
}
#[derive(Default, Clone)]
pub struct TypeTree {
    pub nodes: Vec<TypeTreeNode>,
    pub string_buffer: Vec<u8>,
}
