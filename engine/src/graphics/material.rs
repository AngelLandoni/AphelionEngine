use ahash::HashMap;
use downcast_rs::Downcast;

use crate::scene::assets::AssetResourceID;

pub trait Material: Downcast + Send + Sync {}

/// Represent all the possible types of `Material` types.
pub enum MaterialTexture {
    Diffuse,
}

impl MaterialTexture {
    /// Returns the texture type in form of string.
    pub fn as_string(&self) -> String {
        match self {
            MaterialTexture::Diffuse => "tex_diffuse".to_owned(),
        }
    }
}

/// A convenient type used to describe the store location of all the
/// textures.
type MaterialTextures = HashMap<String, AssetResourceID>;

/// A representation of a default `Material` stored in GPU.
pub struct DefaultMaterial {
    /// Contains all the available textures for the material.
    pub textures: MaterialTextures,
}

impl Material for DefaultMaterial {

}
