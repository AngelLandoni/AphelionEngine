use std::ops::Deref;

use ahash::HashMap;
use downcast_rs::Downcast;
use shipyard::Component;

use crate::scene::assets::AssetResourceID;

/// Represents a material component associated with an asset resource ID.
///
/// This component stores the ID of the material associated with an entity.
#[derive(Component)]
pub struct MaterialComponent(pub(crate) AssetResourceID);

impl MaterialComponent {
    /// Creates a new instance of `MaterialComponent` with the given asset resource ID.
    pub fn new(id: AssetResourceID) -> Self {
        Self(id)
    }
}

impl Deref for MaterialComponent {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Trait representing a material.
///
/// Any struct implementing this trait can be considered a material.
pub trait Material: Downcast + Send + Sync {}

/// Represents all the possible types of textures associated with a material.
pub enum MaterialTexture {
    /// Diffuse texture.
    Diffuse,
}

impl MaterialTexture {
    /// Returns the texture type as a string.
    pub fn as_string(&self) -> String {
        match self {
            MaterialTexture::Diffuse => "tex_diffuse".to_owned(),
        }
    }
}

/// A convenient type used to describe the storage location of all the textures associated with a material.
type MaterialTextures = HashMap<String, AssetResourceID>;

/// Represents a default material stored in the GPU.
///
/// This material contains a collection of textures.
pub struct DefaultMaterial {
    /// Contains all the available textures for the material.
    pub textures: MaterialTextures,
}

impl DefaultMaterial {
    /// Returns the ID associated with the default material.
    pub fn id() -> AssetResourceID {
        "default_material".to_owned()
    }
}

impl Material for DefaultMaterial {}
