use crate::asset::SerializedFile;
use crate::error::{UnityError, UnityResult};
use crate::reader::{ByteOrder, Reader};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(PartialEq)]
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
    #[allow(dead_code)]
    fn from_magic_num(num: u32) -> UnityResult<Self> {
        let ret = match num {
            0x3f => Self::CompressionTypeMask,
            0x40 => Self::BlocksAndDirectoryInfoCombined,
            0x80 => Self::BlocksInfoAtTheEnd,
            0x100 => Self::OldWebPluginCompatibility,
            0x200 => Self::BlockInfoNeedPaddingAtStart,
            _ => return Err(UnityError::InvalidValue),
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

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CompressionType {
    None,
    Lzma,
    Lz4,
    Lz4HC,
    LzInv,
}

impl CompressionType {
    pub fn from_magic_num(num: u32) -> UnityResult<Self> {
        let ret = match num {
            0 => Self::None,
            1 => Self::Lzma,
            2 => Self::Lz4,
            3 => Self::Lz4HC,
            4 => Self::LzInv,
            _ => return Err(UnityError::InvalidValue),
        };
        Ok(ret)
    }
}

#[derive(Debug)]
pub struct BundleHead {
    pub signature: String,
    pub version: u32,
    pub unity_version: String,
    pub unity_revision: String,
    pub size: u64,
    pub compressed_blocks_info_size: u32,
    pub uncompressed_blocks_info_size: u32,
    pub flags: u32,
}

impl Default for BundleHead {
    fn default() -> Self {
        Self::new()
    }
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

#[derive(Debug)]
pub struct StorageBlock {
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub flags: u16,
}

#[derive(Debug)]
pub struct Node {
    pub offset: i64,
    pub size: i64,
    pub flags: u32,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct LoadedFile {
    pub path: String,
    pub name: String,
    pub data: Arc<Vec<u8>>,
}

#[derive(Default)]
pub struct LoadOutput<T> {
    pub metadata: T,
    pub serialized_files: Vec<SerializedFile>,
    pub loaded_files: Vec<LoadedFile>,
}

pub trait FileLoader {
    type MetaData;

    fn name(&self) -> &str;
    fn load(&self, data: &[u8]) -> UnityResult<LoadOutput<Self::MetaData>>;
}

pub struct BundleFileMetaData {
    pub head: BundleHead,
    pub serialized_files: Vec<String>,
    pub loaded_files: Vec<String>,
}

pub struct BundleFileLoader;

impl BundleFileLoader {
    pub fn check(&self, data: &[u8]) -> bool {
        let mut r = Reader::new(data, ByteOrder::Big);
        let signature = r.read_u8_list_util_null_with_limit(20);
        matches!(signature.as_slice(), b"UnityWeb" | b"UnityRaw" | b"UnityArchive" | b"UnityFS")
    }

    pub fn read_header(&self, r: &mut Reader) -> UnityResult<BundleHead> {
        let signature = r.read_string_util_null_with_limit(20);
        if !matches!(signature.as_bytes(), b"UnityWeb" | b"UnityRaw" | b"UnityArchive" | b"UnityFS") {
            return Err(UnityError::FileTypeMissMatch("BundleFile".to_string()));
        }

        let version = r.read_u32()?;
        let unity_version = r.read_string_util_null()?;
        let unity_revision = r.read_string_util_null()?;

        let size = r.read_i64()? as u64;
        let compressed_blocks_info_size = r.read_u32()?;
        let uncompressed_blocks_info_size = r.read_u32()?;
        let flags = r.read_u32()?;
        if signature != "UnityFS" {
            r.read_u8()?;
        }
        Ok(BundleHead {
            signature,
            version,
            unity_version,
            unity_revision,
            size,
            compressed_blocks_info_size,
            uncompressed_blocks_info_size,
            flags,
        })
    }

    pub fn read_blocks_info_and_directory(&self, r: &mut Reader, header: &BundleHead) -> UnityResult<(Vec<StorageBlock>, Vec<Node>)> {
        let block_info_bytes: Vec<u8>;
        if header.version >= 7 {
            r.align(16)?;
        }
        let offset = r.get_offset();
        if header.flags & ArchiveFlags::BlocksInfoAtTheEnd as u32 != 0 {
            r.set_offset(r.len() - header.compressed_blocks_info_size as usize)?;
            block_info_bytes = r.read_u8_list(header.compressed_blocks_info_size as usize)?;
            r.set_offset(offset)?;
        } else {
            block_info_bytes = r.read_u8_list(header.compressed_blocks_info_size as usize)?;
        }
        let uncompressed_size = header.uncompressed_blocks_info_size;
        let compressed_type = CompressionType::from_magic_num(header.flags & ArchiveFlags::CompressionTypeMask as u32)?;
        let block_info_uncompressed_bytes = match compressed_type {
            CompressionType::None => block_info_bytes,
            CompressionType::Lzma => return Err(UnityError::Unimplemented),
            CompressionType::Lz4 | CompressionType::Lz4HC => lz4_flex::decompress(&block_info_bytes, uncompressed_size as usize)?,
            CompressionType::LzInv => return Err(UnityError::Unimplemented),
        };
        let mut block_info_reader = Reader::new(&block_info_uncompressed_bytes, ByteOrder::Big);
        let _uncompressed_data_hash = block_info_reader.read_u8_slice(16)?;
        let block_info_count = block_info_reader.read_i32()?;

        let mut block_infos = Vec::new();

        for _ in 0..block_info_count {
            let s = StorageBlock {
                uncompressed_size: block_info_reader.read_u32()?,
                compressed_size: block_info_reader.read_u32()?,
                flags: block_info_reader.read_u16()?,
            };
            block_infos.push(s)
        }

        let node_count = block_info_reader.read_i32()?;

        let mut nodes = Vec::new();

        for _ in 0..node_count {
            let n = Node {
                offset: block_info_reader.read_i64()?,
                size: block_info_reader.read_i64()?,
                flags: block_info_reader.read_u32()?,
                path: block_info_reader.read_string_util_null()?,
            };
            nodes.push(n)
        }
        if header.flags & ArchiveFlags::BlockInfoNeedPaddingAtStart as u32 != 0 {
            r.align(16)?;
        }
        Ok((block_infos, nodes))
    }

    pub fn read_blocks(&self, r: &mut Reader, block_infos: &[StorageBlock]) -> UnityResult<Vec<u8>> {
        let mut result = Vec::new();
        for block_info in block_infos {
            let compress_type = CompressionType::from_magic_num((block_info.flags & StorageBlockFlags::CompressionTypeMask as u16) as u32)?;
            match compress_type {
                CompressionType::None => {
                    result.extend_from_slice(r.read_u8_slice(block_info.compressed_size as usize)?);
                }
                CompressionType::Lzma => {
                    let in_buf = r.read_u8_slice(block_info.compressed_size as usize)?;
                    let mut in_buf_new = Vec::new();
                    in_buf_new.extend_from_slice(&in_buf[..5]);
                    in_buf_new.extend_from_slice(&(block_info.uncompressed_size as u64).to_le_bytes());
                    in_buf_new.extend_from_slice(&in_buf[5..]);
                    let mut out_buf = std::io::Cursor::new(&mut result);
                    lzma_rs::lzma_decompress(&mut std::io::Cursor::new(in_buf_new), &mut out_buf)?;
                }
                CompressionType::Lz4 | CompressionType::Lz4HC | CompressionType::LzInv => {
                    let compressed_size = block_info.compressed_size;
                    let compressed_bytes = r.read_u8_slice(compressed_size as usize)?;
                    let uncompressed_size = block_info.uncompressed_size;

                    let uncompressed_bytes = if compress_type == CompressionType::LzInv {
                        let mut buf = compressed_bytes.to_vec();
                        lz4_inv::swap(&mut buf, uncompressed_size as usize)?;
                        lz4_flex::decompress(&buf, uncompressed_size as usize)?
                    } else {
                        lz4_flex::decompress(compressed_bytes, uncompressed_size as usize)?
                    };

                    result.extend_from_slice(&uncompressed_bytes);
                }
            }
        }
        Ok(result)
    }

    pub fn read_files(&self, data: &[u8], nodes: &[Node]) -> UnityResult<Vec<LoadedFile>> {
        let mut r = Reader::new(data, ByteOrder::Big);

        let mut files = Vec::with_capacity(nodes.len());
        for node in nodes {
            r.set_offset(node.offset as usize)?;
            let data = r.read_u8_list(node.size as usize)?;

            let name = std::path::Path::file_name(node.path.as_ref()).map(|x| x.to_string_lossy().to_string()).unwrap_or_else(|| node.path.to_string());
            files.push(LoadedFile {
                path: node.path.to_string(),
                name,
                data: Arc::from(data),
            })
        }
        Ok(files)
    }

    pub fn load_assets(&self, head: BundleHead, files: &[LoadedFile]) -> UnityResult<LoadOutput<BundleFileMetaData>> {
        let mut serialized_files = Vec::new();
        let mut loaded_files = Vec::new();

        let mut metadata = BundleFileMetaData {
            head,
            serialized_files: Vec::new(),
            loaded_files: Vec::new(),
        };
        for file in files {
            if let Ok(FileType::AssetsFile) = check_file_type(&file.data) {
                metadata.serialized_files.push(file.path.clone());
                let serialized_file = SerializedFile::new(Arc::clone(&file.data), &file.path)?;
                serialized_files.push(serialized_file);
            } else {
                metadata.loaded_files.push(file.path.clone());
                loaded_files.push(file.clone());
            }
        }
        Ok(LoadOutput { metadata, serialized_files, loaded_files })
    }
}

impl FileLoader for BundleFileLoader {
    type MetaData = BundleFileMetaData;

    fn name(&self) -> &str {
        "BundleFileLoader"
    }

    fn load(&self, data: &[u8]) -> UnityResult<LoadOutput<Self::MetaData>> {
        if !self.check(data) {
            return Err(UnityError::FileTypeMissMatch("BundleFile".to_string()));
        }

        let mut r = Reader::new(data, ByteOrder::Big);
        let head = self.read_header(&mut r)?;
        if head.signature != "UnityFS" {
            return Err(UnityError::UnsupportFileType("UnityWeb or UnityRaw".into()));
        }

        let (block_infos, nodes) = self.read_blocks_info_and_directory(&mut r, &head)?;
        let block_datas = self.read_blocks(&mut r, &block_infos)?;
        let files = self.read_files(&block_datas, &nodes)?;
        let output = self.load_assets(head, &files)?;

        Ok(output)
    }
}

pub fn check_file_type(data: &[u8]) -> UnityResult<FileType> {
    fn is_serialized_file(r: &mut Reader) -> UnityResult<bool> {
        if r.len() < 20 {
            return Ok(false);
        }
        let mut _metadata_size = r.read_u32()?;
        let mut file_size = r.read_u32()? as i64;
        let version = r.read_u32()?;
        let mut data_offset = r.read_u32()? as i64;
        let _endian = r.read_u8()?;
        let _reserved = r.read_u8_array::<3>()?;
        if version >= 22 {
            if r.len() < 48 {
                return Ok(false);
            }
            _metadata_size = r.read_u32()?;
            file_size = r.read_i64()?;
            data_offset = r.read_i64()?;
        }
        if r.len() != file_size as usize {
            return Ok(false);
        }
        if data_offset > file_size {
            return Ok(false);
        }
        Ok(true)
    }
    if data.len() < 20 {
        return Ok(FileType::ResourceFile);
    }
    let gzip_magic = [0x1f, 0x8b];
    let brotli_magic = [0x62, 0x72, 0x6F, 0x74, 0x6C, 0x69];
    let zip_magic = [0x50, 0x4B, 0x03, 0x04];
    let zip_spanned_magic = [0x50, 0x4B, 0x07, 0x08];
    let mut r = Reader::new(data, ByteOrder::Big);
    let signature = r.read_u8_list_util_null_with_limit(20);
    match signature.as_slice() {
        b"UnityWeb" | b"UnityRaw" | b"UnityArchive" | b"UnityFS" => Ok(FileType::BundleFile),
        b"UnityWebData1.0" => Ok(FileType::WebFile),
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
mod lz4_inv {
    use crate::{UnityError, UnityResult};
    pub fn swap(buf: &mut [u8], uncompressed_size: usize) -> UnityResult<()> {
        let mut cmp_pos = 0;
        let mut dec_pos = 0;
        loop {
            let (mut enc_cnt, mut lit_cnt) = swap_literal_token(buf, &mut cmp_pos)?;
            lit_cnt = get_length(lit_cnt, buf, &mut cmp_pos)?;
            cmp_pos += lit_cnt;
            dec_pos += lit_cnt;

            if cmp_pos >= buf.len() {
                break;
            }

            swap_chunk_end(buf, &mut cmp_pos)?;
            enc_cnt = get_length(enc_cnt, buf, &mut cmp_pos)? + 4;
            dec_pos += enc_cnt;

            if cmp_pos >= buf.len() || dec_pos >= uncompressed_size {
                break;
            }
        }
        Ok(())
    }

    fn swap_literal_token(buf: &mut [u8], idx: &mut usize) -> UnityResult<(usize, usize)> {
        if *idx >= buf.len() {
            return Err(UnityError::Eof);
        }
        let enc_count = (buf[*idx] >> 4) & 0xf;
        let lit_count = buf[*idx] & 0xf;
        buf[*idx] = enc_count + (lit_count << 4);
        *idx += 1;
        Ok((enc_count as usize, lit_count as usize))
    }

    fn swap_chunk_end(buf: &mut [u8], idx: &mut usize) -> UnityResult<usize> {
        if *idx + 1 >= buf.len() {
            return Err(UnityError::Eof);
        }
        let high = (buf[*idx] as u16) << 8;
        *idx += 1;
        let low = buf[*idx] as u16;
        buf.swap(*idx - 1, *idx);
        *idx += 1;
        Ok((high | low) as usize)
    }

    fn get_length(mut length: usize, buf: &[u8], idx: &mut usize) -> UnityResult<usize> {
        if length != 0xf {
            return Ok(length);
        }

        let mut sum;
        loop {
            if *idx >= buf.len() {
                return Err(UnityError::Eof);
            }
            sum = buf[*idx];
            *idx += 1;
            length += sum as usize;
            if sum != 0xff {
                break;
            }
        }
        Ok(length)
    }
}
