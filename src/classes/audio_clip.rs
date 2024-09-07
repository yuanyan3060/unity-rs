use std::collections::HashMap;

use num_enum::FromPrimitive;

use crate::classes::FromObject;
use crate::env::Object;
use crate::error::UnityResult;
use crate::reader::{ByteOrder, Reader};
use crate::UnityError;

#[derive(Debug, Eq, PartialEq, FromPrimitive, Clone, Copy)]
#[repr(i32)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum AudioCompressionFormat {
    #[num_enum(default)]
    UnknownType = -1,
    PCM = 0,
    Vorbis = 1,
    ADPCM = 2,
    MP3 = 3,
    PSMVAG = 4,
    HEVAG = 5,
    XMA = 6,
    AAC = 7,
    GCADPCM = 8,
    ATRAC9 = 9,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, Clone, Copy)]
#[repr(i32)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum FMODSoundType {
    #[num_enum(default)]
    UNKNOWN = 0,
    ACC = 1,
    AIFF = 2,
    ASF = 3,
    AT3 = 4,
    CDDA = 5,
    DLS = 6,
    FLAC = 7,
    FSB = 8,
    GCADPCM = 9,
    IT = 10,
    MIDI = 11,
    MOD = 12,
    MPEG = 13,
    OGGVORBIS = 14,
    PLAYLIST = 15,
    RAW = 16,
    S3M = 17,
    SF2 = 18,
    USER = 19,
    WAV = 20,
    XM = 21,
    XMA = 22,
    VAG = 23,
    AUDIOQUEUE = 24,
    XWMA = 25,
    BCWAV = 26,
    AT9 = 27,
    VORBIS = 28,
    MEDIA_FOUNDATION = 29,
}

pub enum AudioClipMeta {
    Low {
        format: i32,
        typ: FMODSoundType,
        is_3d: bool,
        use_hardware: bool,
    },
    High {
        load_type: i32,
        channels: i32,
        frequency: i32,
        bits_per_sample: i32,
        length: f32,
        is_tracker_format: bool,
        subsound_index: i32,
        preload_audio_data: bool,
        load_in_background: bool,
        legacy_3d: bool,
        compression_format: AudioCompressionFormat,
    },
}

pub struct AudioClip {
    pub name: String,
    pub meta: AudioClipMeta,
    pub source: Option<String>,
    pub offset: Option<i64>,
    pub size: i64,
    pub data: Vec<u8>,
}

impl<'a> FromObject<'a> for AudioClip {
    fn load(object: &Object) -> UnityResult<Self> {
        let mut r = object.info.get_reader();
        let name = r.read_aligned_string()?;
        let meta: AudioClipMeta;
        let size: i64;
        let mut offset: Option<i64> = None;
        let mut source: Option<String> = None;
        if object.asset.version[0] < 5 {
            meta = AudioClipMeta::Low {
                format: r.read_i32()?,
                typ: r.read_i32()?.into(),
                is_3d: r.read_bool()?,
                use_hardware: r.read_bool()?,
            };
            r.align(4)?;
            if object.asset.version_greater_or_equal(&[3, 2]) {
                let _stream = r.read_i32()?;
                size = r.read_i32()? as i64;
                let tsize = if size % 4 != 0 { size + 4 - size % 4 } else { size };
                if r.len() - r.get_offset() != tsize as usize {
                    offset = Some(r.read_u32()? as i64);
                    source = Some(object.asset.path.clone()); // 可能与unitypy不同
                }
            } else {
                size = r.read_i32()? as i64;
            }
        } else {
            meta = AudioClipMeta::High {
                load_type: r.read_i32()?,
                channels: r.read_i32()?,
                frequency: r.read_i32()?,
                bits_per_sample: r.read_i32()?,
                length: r.read_f32()?,
                is_tracker_format: {
                    let is_tracker_format = r.read_bool()?;
                    r.align(4)?;
                    is_tracker_format
                },
                subsound_index: r.read_i32()?,
                preload_audio_data: r.read_bool()?,
                load_in_background: r.read_bool()?,
                legacy_3d: {
                    let legacy_3d = r.read_bool()?;
                    r.align(4)?;
                    source = Some(r.read_aligned_string()?);
                    offset = Some(r.read_i64()?);
                    size = r.read_i64()?;
                    legacy_3d
                },
                compression_format: { r.read_i32()?.into() },
            }
        }
        let data = match (source.as_deref(), offset) {
            (Some(source), Some(offset)) => {
                let mut data = None;
                let path = source.split('/').last().ok_or(UnityError::InvalidValue)?;
                for i in 0..object.bundle.nodes.len() {
                    let node = &object.bundle.nodes[i];
                    if node.path != path {
                        continue;
                    }
                    let file = &object.bundle.files[i];
                    let mut r = Reader::new(file.as_slice(), ByteOrder::Big);
                    r.set_offset(offset as usize)?;
                    data = Some(r.read_u8_list(size as usize)?);
                    break;
                }
                match data {
                    Some(data) => data,
                    None => return Err(UnityError::CustomError("can not find resource".to_string())),
                }
            }
            _ => r.read_u8_list(size as usize)?,
        };
        Ok(Self { name, meta, source, offset, size, data })
    }

    fn class() -> super::ClassID {
        super::ClassID::AudioClip
    }
}

impl AudioClip {
    pub fn samples(&self) -> UnityResult<HashMap<String, Vec<u8>>> {
        let mut ret = HashMap::new();
        match self.data.as_slice() {
            [b'O', b'g', b'g', b'S', ..] => {
                ret.insert(format!("{}.ogg", self.name), self.data.clone());
            }
            [b'R', b'I', b'F', b'F', ..] => {
                ret.insert(format!("{}.wav", self.name), self.data.clone());
            }
            [_, _, _, _, b'f', b't', b'y', b'p', ..] => {
                ret.insert(format!("{}.m4a", self.name), self.data.clone());
            }
            _ => {
                ret.insert(format!("{}.fsb", self.name), self.data.clone()); // 暂时先不解码fbs了（
            }
        }
        Ok(ret)
    }
}
