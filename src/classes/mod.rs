mod animation_clip;
mod audio_clip;
mod component;
mod game_object;
mod id;
mod material;
mod mesh;
mod mesh_renderer;
mod mono_behaviour;
mod mono_script;
mod pptr;
mod renderer;
mod shader;
mod sprite;
mod sprite_atlas;
mod text_asset;
mod texture2d;
mod transform;

use crate::error::UnityResult;
pub use id::ClassID;

use crate::env::Object;
pub use audio_clip::AudioClip;
pub use component::Component;
pub use game_object::GameObject;
pub use material::Material;
pub use mesh::Mesh;
pub use mesh_renderer::MeshRenderer;
pub use mono_behaviour::MonoBehaviour;
pub use mono_script::MonoScript;
pub use renderer::Renderer;
pub use sprite::Sprite;
pub use text_asset::TextAsset;
pub use texture2d::Texture2D;
pub use transform::Transform;

pub trait FromObject<'a>
where
    Self: Sized,
{
    fn load(object: &'a Object<'a>) -> UnityResult<Self>;
    fn class() -> ClassID;
}
