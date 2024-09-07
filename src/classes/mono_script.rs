use crate::classes::FromObject;
use crate::env::Object;
use crate::error::UnityResult;

pub struct MonoScript {
    pub name: String,
    pub class_name: String,
    pub namespace: Option<String>,
    pub assembly_name: String,
}

impl<'a> FromObject<'a> for MonoScript {
    fn load(object: &Object) -> UnityResult<Self> {
        let version = object.info.version;
        let mut r = object.info.get_reader();
        let name = r.read_aligned_string()?;
        if version[0] > 3 || (version[0] == 3 && version[1] >= 4) {
            let _execution_order = r.read_i32()?;
        }
        if version[0] < 5 {
            let _properties_hash = r.read_u32()?;
        } else {
            let _properties_hash = r.read_u8_array::<16>()?;
        }
        if version[0] < 3 {
            let _path_name = r.read_aligned_string()?;
        }
        let class_name = r.read_aligned_string()?;
        let namespace = if version[0] >= 3 { Some(r.read_aligned_string()?) } else { None };
        let assembly_name = r.read_aligned_string()?;
        if version[0] < 2018 || (version[0] == 2018 && version[1] < 2) {
            let _is_editor_script = r.read_bool()?;
        }
        Ok(Self { name, class_name, namespace, assembly_name })
    }

    fn class() -> super::ClassID {
        super::ClassID::MonoScript
    }
}
