mod animation_clip;
mod audio_clip;
mod component;
mod game_object;
mod id;
mod mesh;
mod mono_behaviour;
mod mono_script;
mod pptr;
mod sprite;
mod sprite_atlas;
mod text_asset;
mod texture2d;
mod material;
mod shader;
mod transform;
mod renderer;
mod mesh_renderer;

use crate::error::UnityResult;
pub use id::ClassID;

use crate::env::Object;
pub use audio_clip::AudioClip;
pub use mesh::Mesh;
pub use mono_behaviour::MonoBehaviour;
pub use mono_script::MonoScript;
pub use sprite::Sprite;
pub use text_asset::TextAsset;
pub use texture2d::Texture2D;
pub use material::Material;
pub use transform::Transform;
pub use component::Component;
pub use game_object::GameObject;
pub use renderer::Renderer;
pub use mesh_renderer::MeshRenderer;

pub trait FromObject<'a>
where
    Self: Sized,
{
    fn load(object: &'a Object<'a>) -> UnityResult<Self>;
    fn class() -> ClassID;
}
