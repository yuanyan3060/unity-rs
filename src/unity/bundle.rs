#![allow(unused)]
use std::collections::HashMap;

use bytes::Bytes;

use super::{asset::Asset, object::ObjectInfo, ByteOrder, Error, Reader, Result};
pub enum FileType {
    AssetsFile,
    BundleFile,
    WebFile,
    ResourceFile,
    GZipFile,
    BrotliFile,
    ZipFile,
}

#[derive(Debug)]
#[repr(u32)]
pub enum ArchiveFlags {
    CompressionTypeMask = 0x3f,
    BlocksAndDirectoryInfoCombined = 0x40,
    BlocksInfoAtTheEnd = 0x80,
    OldWebPluginCompatibility = 0x100,
    BlockInfoNeedPaddingAtStart = 0x200,
}

impl ArchiveFlags {
    fn from_magic_num(num: u32) -> Result<Self> {
        dbg!("archive flag:{}", num);
        let ret = match num {
            0x3f => Self::CompressionTypeMask,
            0x40 => Self::BlocksAndDirectoryInfoCombined,
            0x80 => Self::BlocksInfoAtTheEnd,
            0x100 => Self::OldWebPluginCompatibility,
            0x200 => Self::BlockInfoNeedPaddingAtStart,
            _ => return Err(Error::InvalidValue),
        };
        Ok(ret)
    }
}

#[derive(Debug)]
#[repr(u16)]
pub enum StorageBlockFlags {
    CompressionTypeMask = 0x3f,
    Streamed = 0x40,
}

pub enum CompressionType {
    None,
    Lzma,
    Lz4,
    Lz4HC,
    Lzham,
}

impl CompressionType {
    pub fn from_magic_num(num: u32) -> Result<Self> {
        let ret = match num {
            0 => Self::None,
            1 => Self::Lzma,
            2 => Self::Lz4,
            3 => Self::Lz4HC,
            4 => Self::Lzham,
            _ => return Err(Error::InvalidValue),
        };
        Ok(ret)
    }
}

#[derive(Debug)]
pub struct BundleHead {
    signature: String,
    version: u32,
    unity_version: String,
    unity_revision: String,
    size: u64,
    compressed_blocks_info_size: u32,
    uncompressed_blocks_info_size: u32,
    flags: u32,
}

impl BundleHead {
    pub fn new() -> Self {
        BundleHead {
            signature: String::new(),
            version: 0,
            unity_version: String::new(),
            unity_revision: String::new(),
            size: 0,
            compressed_blocks_info_size: 0,
            uncompressed_blocks_info_size: 0,
            flags: 0,
        }
    }
}

pub struct StorageBlock {
    compressed_size: u32,
    uncompressed_size: u32,
    flags: u16,
}

pub struct Node {
    pub offset: i64,
    pub size: i64,
    pub flags: u32,
    pub path: String,
}

pub struct AssetBundle {
    header: BundleHead,
    block_infos: Vec<StorageBlock>,
    pub nodes: Vec<Node>,
    pub files: Vec<Bytes>,
}

impl AssetBundle {
    pub fn load(data: Bytes) -> Result<Self> {
        let mut r = Reader::new(data, ByteOrder::Big);
        let signature = r.read_string_utill_null()?;
        let version = r.read_u32()?;
        let unity_version = r.read_string_utill_null()?;
        let unity_revision = r.read_string_utill_null()?;
        let mut ret = Self {
            header: BundleHead {
                signature,
                version,
                unity_version,
                unity_revision,
                size: 0,
                compressed_blocks_info_size: 0,
                uncompressed_blocks_info_size: 0,
                flags: 0,
            },
            block_infos: Vec::new(),
            nodes: Vec::new(),
            files: Vec::new(),
        };
        match ret.header.signature.as_str() {
            "UnityWeb" | "UnityRaw" => {
                if version != 6 {
                    ret.read_header_and_blocks_info(&mut r)?;
                }
                todo!()
            }
            "UnityFS" => {
                ret.read_header(&mut r)?;
                ret.read_blocks_info_and_directory(&mut r)?;
                let blocks_data = ret.read_blocks(&mut r)?;
                ret.read_files(blocks_data)?;
            }
            _ => {
                unimplemented!()
            }
        }

        Ok(ret)
    }
    /*pub fn assets(&self) -> Result<Vec<Asset>> {
        let mut ret = Vec::new();
        for f in &self.files {
            if let FileType::AssetsFile = Self::check_file_type(f.clone())? {
                let a = Asset::new(f.clone())?;
                ret.push(a);
            }
        }
        Ok(ret)
    }*/

    /*pub fn objects(&self) -> Result<HashMap<i64, ObjectInfo>> {
        let mut ret = HashMap::new();
        let a = self.assets()?;
        for i in a {
            for j in i.objects_info {
                ret.insert(j.path_id, j);
            }
        }
        Ok(ret)
    }*/
    fn read_header_and_blocks_info(&mut self, r: &mut Reader) -> Result<()> {
        if self.header.version >= 4 {
            let hash = r.read_u8_list(16)?;
            let crc = r.read_i32()?;
        }
        let mini_num_streamed_bytes = r.read_u32()?;
        self.header.size = r.read_u32()? as u64;
        Ok(())
    }

    fn read_header(&mut self, r: &mut Reader) -> Result<()> {
        self.header.size = r.read_i64()? as u64;
        self.header.compressed_blocks_info_size = r.read_u32()?;
        self.header.uncompressed_blocks_info_size = r.read_u32()?;
        self.header.flags = r.read_u32()?;
        if self.header.signature != "UnityFS" {
            r.read_u8()?;
        }
        Ok(())
    }

    fn read_blocks_info_and_directory(&mut self, r: &mut Reader) -> Result<()> {
        let block_info_bytes: Bytes;
        if self.header.version >= 7 {
            r.align(16)?;
        }
        let offset = r.get_offset();
        if self.header.flags & ArchiveFlags::BlocksInfoAtTheEnd as u32 != 0 {
            r.set_offset(r.get_size() - self.header.compressed_blocks_info_size as usize)?;
            block_info_bytes = r.read_u8_slice(self.header.compressed_blocks_info_size as usize)?;
            r.set_offset(offset)?;
        } else {
            block_info_bytes = r.read_u8_slice(self.header.compressed_blocks_info_size as usize)?;
        }
        // MemoryStream blocksInfoUncompresseddStream;
        let uncompressed_size = self.header.uncompressed_blocks_info_size;
        let compressed_type = CompressionType::from_magic_num(
            self.header.flags & ArchiveFlags::CompressionTypeMask as u32,
        )?;
        let block_info_uncompressed_bytes = match compressed_type {
            CompressionType::None => block_info_bytes.to_vec(),
            CompressionType::Lzma => todo!(),
            CompressionType::Lz4 | CompressionType::Lz4HC => {
                lz4_flex::decompress(&block_info_bytes, uncompressed_size as usize)
                    .expect("uncompresse error")
            }
            CompressionType::Lzham => todo!(),
        };
        let mut block_info_reader = Reader::new(Bytes::from(block_info_uncompressed_bytes), ByteOrder::Big);
        let uncompressed_data_hash = block_info_reader.read_u8_list(16)?;
        let block_info_count = block_info_reader.read_i32()?;
        for _ in 0..block_info_count {
            let s = StorageBlock {
                uncompressed_size: block_info_reader.read_u32()?,
                compressed_size: block_info_reader.read_u32()?,
                flags: block_info_reader.read_u16()?,
            };
            self.block_infos.push(s)
        }
        let node_count = block_info_reader.read_i32()?;
        for _ in 0..node_count {
            let n = Node {
                offset: block_info_reader.read_i64()?,
                size: block_info_reader.read_i64()?,
                flags: block_info_reader.read_u32()?,
                path: block_info_reader.read_string_utill_null()?,
            };
            self.nodes.push(n)
        }
        if self.header.flags & ArchiveFlags::BlockInfoNeedPaddingAtStart as u32 != 0 {
            r.align(16)?;
        }
        Ok(())
    }

    fn read_blocks(&self, r: &mut Reader) -> Result<Bytes> {
        let mut result = Vec::new();
        for block_info in &self.block_infos {
            let compress_type = CompressionType::from_magic_num(
                (block_info.flags & StorageBlockFlags::CompressionTypeMask as u16) as u32,
            )?;
            match compress_type {
                CompressionType::None => {
                    result.extend_from_slice(&r.read_u8_slice(block_info.compressed_size as usize)?);
                },
                CompressionType::Lzma => todo!(),
                CompressionType::Lz4 => todo!(),
                CompressionType::Lz4HC | CompressionType::Lzham => {
                    let compressed_size = block_info.compressed_size;
                    let compressed_bytes = r.read_u8_slice(compressed_size as usize)?;
                    let uncompressed_size = block_info.uncompressed_size;
                    let uncompressed_bytes =
                        lz4_flex::decompress(&compressed_bytes, uncompressed_size as usize)
                            .expect("uncompresse error");
                    result.extend_from_slice(&uncompressed_bytes);
                }
            }
        }
        Ok(Bytes::from(result))
    }

    fn read_files(&mut self, data: Bytes) -> Result<()> {
        let mut r = Reader::new(data, ByteOrder::Big);
        for node in &self.nodes {
            r.set_offset(node.offset as usize)?;
            let file = r.read_u8_list(node.size as usize)?;
            self.files.push(file)
        }
        Ok(())
    }

    pub(super) fn check_file_type(data: Bytes) -> Result<FileType> {
        fn is_serialized_file(r: &mut Reader) -> Result<bool> {
            if r.get_size() < 20 {
                return Ok(false);
            }
            let mut metadata_size = r.read_u32()?;
            let mut file_size = r.read_u32()? as i64;
            let version = r.read_u32()?;
            let mut data_offset = r.read_u32()? as i64;
            let endianess = r.read_u8()?;
            let reserved = r.read_u8_array::<3>()?;
            if version >= 22 {
                if r.get_size() < 48 {
                    return Ok(false);
                }
                metadata_size = r.read_u32()?;
                file_size = r.read_i64()?;
                data_offset = r.read_i64()?;
            }
            if r.get_size() != file_size as usize {
                return Ok(false);
            }
            if data_offset > file_size {
                return Ok(false);
            }
            Ok(true)
        }
        let gzip_magic = [0x1f, 0x8b];
        let brotli_magic = [0x62, 0x72, 0x6F, 0x74, 0x6C, 0x69];
        let zip_magic = [0x50, 0x4B, 0x03, 0x04];
        let zip_spanned_magic = [0x50, 0x4B, 0x07, 0x08];
        let mut r = Reader::new(data, ByteOrder::Big);
        let signature = r.read_string_utill_null_limit(20)?;
        match signature.as_str() {
            "UnityWeb" | "UnityRaw" | "UnityArchive" | "UnityFS" => Ok(FileType::BundleFile),
            "UnityWebData1.0" => Ok(FileType::WebFile),
            _ => {
                let magic: [u8; 2] = r.read_u8_array()?;
                r.set_offset(0)?;
                if magic == gzip_magic {
                    return Ok(FileType::GZipFile);
                }
                r.set_offset(0x20)?;
                let magic: [u8; 6] = r.read_u8_array()?;
                r.set_offset(0)?;
                if magic == brotli_magic {
                    return Ok(FileType::BrotliFile);
                }
                if is_serialized_file(&mut r)? {
                    return Ok(FileType::AssetsFile);
                }
                let magic: [u8; 4] = r.read_u8_array()?;
                r.set_offset(0)?;
                if magic == zip_magic || magic == zip_spanned_magic {
                    return Ok(FileType::ZipFile);
                }
                Ok(FileType::ResourceFile)
            }
        }
    }

    fn read_block_infos_and_nodes(&mut self, r: &mut Reader) -> Result<()> {
        let blocks_info_bytes: Bytes;
        let mut blocks_info_uncompressed_bytes = Vec::new();
        if self.header.version >= 7 {
            r.align(16)?;
        }
        let origin_offset: usize;
        if self.header.flags & (ArchiveFlags::BlocksInfoAtTheEnd as u32) != 0 {
            origin_offset = r.get_offset();
            let offset = r.get_size() - self.header.compressed_blocks_info_size as usize;
            r.set_offset(offset)?;
            let length = self.header.compressed_blocks_info_size as usize;
            blocks_info_bytes = r.read_u8_slice(length)?;
        } else {
            let length = self.header.compressed_blocks_info_size as usize;
            origin_offset = r.get_offset();
            blocks_info_bytes = r.read_u8_slice(length)?;
        }
        let compressed_num = self.header.flags & ArchiveFlags::CompressionTypeMask as u32;
        match CompressionType::from_magic_num(compressed_num)? {
            CompressionType::None => {
                blocks_info_uncompressed_bytes = blocks_info_bytes.to_vec();
            }
            CompressionType::Lzma => {}
            CompressionType::Lz4 => todo!(),
            CompressionType::Lz4HC => todo!(),
            CompressionType::Lzham => todo!(),
        };
        r.set_offset(origin_offset)?;

        Ok(())
    }
}
