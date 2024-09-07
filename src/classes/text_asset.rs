use crate::classes::FromObject;
use crate::env::Object;
use crate::error::UnityResult;

pub struct TextAsset {
    pub name: String,
    pub script: Vec<u8>,
    pub path_id: i64,
}

impl<'a> FromObject<'a> for TextAsset {
    fn load(object: &Object) -> UnityResult<Self> {
        let mut r = object.info.get_reader();
        let name = r.read_aligned_string()?;
        let length = r.read_i32()?;
        let script = r.read_u8_list(length as usize)?;
        Ok(Self { name, script, path_id: object.info.path_id })
    }

    fn class() -> super::ClassID {
        super::ClassID::TextAsset
    }
}

impl TextAsset {
    pub fn script_string(&self) -> UnityResult<&str> {
        Ok(std::str::from_utf8(&self.script)?)
    }
}
