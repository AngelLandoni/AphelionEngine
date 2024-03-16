pub mod asset_loader;
pub mod asset_server;
pub mod model;

pub type AssetResourceID = String;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct TextureResourceID(pub(crate) AssetResourceID);
