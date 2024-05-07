mod animation_clip;
mod id;
mod mesh;
mod pptr;
mod sprite;
mod sprite_atlas;
mod text_asset;
mod texture2d;
mod mono_behaviour;
mod game_object;
mod component;
mod mono_script;

use crate::error::UnityResult;
pub use id::ClassID;

use crate::env::Object;
pub use sprite::Sprite;
pub use text_asset::TextAsset;
pub use texture2d::Texture2D;
pub use mono_behaviour::MonoBehaviour;

pub trait FromObject<'a>
where
    Self: Sized,
{
    fn load(object: &'a Object<'a>) -> UnityResult<Self>;
}
