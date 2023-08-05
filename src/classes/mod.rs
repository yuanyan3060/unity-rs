mod animation_clip;
mod decode;
mod id;
mod mesh;
mod pptr;
mod sprite;
mod sprite_atlas;
mod text_asset;
mod texture2d;

use crate::error::UnityResult;
pub use id::ClassID;

use crate::env::Object;
pub use sprite::Sprite;
pub use text_asset::TextAsset;
pub use texture2d::Texture2D;

pub trait FromObject<'a>
where
    Self: Sized,
{
    fn load(object: &'a Object<'a>) -> UnityResult<Self>;
}
