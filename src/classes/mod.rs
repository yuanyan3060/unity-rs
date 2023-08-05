mod id;
mod text_asset;
mod texture2d;
mod decode;
mod sprite;
mod pptr;
mod mesh;
mod animation_clip;
mod sprite_atlas;

pub use id::ClassID;
use crate::error::UnityResult;

pub use texture2d::Texture2D;
pub use text_asset::TextAsset;
pub use sprite::Sprite;
use crate::env::Object;

pub trait FromObject<'a>
    where Self: Sized
{
    fn load(object: &'a Object<'a>) -> UnityResult<Self>;
}