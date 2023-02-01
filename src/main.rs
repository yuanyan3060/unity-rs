#![allow(unused)]
use std::error::Error;

use bytes::Bytes;
use unity::{
    classes::{TextAsset, Texture2D, Sprite},
    ClassIDType, Env,
};

mod unity;

fn main() -> Result<(), Box<dyn Error>> {
    let data: Bytes = std::fs::read("char_1028_texas2.ab")?.into();
    let env = Env::load(data)?;
    for (path_id, object) in &env.objects() {
        if object.class() == ClassIDType::Sprite {
            let s:Sprite = object.read()?;
            println!("{:#?}", s);
            let t = s.rd.texture.get_obj()?;
            t.decode_image()?.save("out.png")?;
            break;
        }
    }
    Ok(())
}
