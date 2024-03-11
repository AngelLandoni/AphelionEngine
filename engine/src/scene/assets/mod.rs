use std::ops::Deref;

pub mod asset_server;

type AssetResourceID = String;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct MeshResourceID(pub(crate) AssetResourceID);

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct TextureResourceID(pub(crate) AssetResourceID);

impl Deref for MeshResourceID {
    type Target = AssetResourceID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
