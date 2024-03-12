pub mod asset_loader;
pub mod asset_server;
pub mod model;

use std::ops::Deref;

type AssetResourceID = String;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct MeshResourceID(pub AssetResourceID);

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct TextureResourceID(pub(crate) AssetResourceID);

impl Deref for MeshResourceID {
    type Target = AssetResourceID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
