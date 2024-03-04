use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
};

use ahash::AHashMap;
use shipyard::Unique;

use crate::graphics::{mesh::Mesh, Texture};

use super::asset_loader::AssetLoader;

type AssetResourceID = &'static str;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct MeshResourceID(pub(crate) AssetResourceID);

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct TextureResourceID(pub(crate) AssetResourceID);

impl Deref for MeshResourceID {
    type Target = AssetResourceID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Conatins all the assets which a `Scene` can use.
#[derive(Unique)]
pub struct AssetServer {
    pub data: Arc<RwLock<AssetServerData>>,

    pub loader: Arc<Mutex<AssetLoader>>,
}

impl Default for AssetServer {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(AssetServerData::default())),
            loader: Arc::new(Mutex::new(AssetLoader::default())),
        }
    }
}

impl AssetServer {
    /// Retrieves a particular `Mesh`.
    pub fn load_mesh(&self, mesh: &MeshResourceID) -> Arc<Mesh> {
        self.data
            .read()
            .expect("Unable to acquire read lock")
            .meshes
            .get(mesh.0)
            .expect("Mesh not found")
            .clone()
    }
}

/// Shipyard requires that `AssetServe` can be used from different threads. The
/// `AssetServerDat` must be protected.
unsafe impl Send for AssetServer {}
unsafe impl Sync for AssetServer {}

#[derive(Default)]
pub struct AssetServerData {
    pub meshes: HashMap<AssetResourceID, Arc<Mesh>>,
    pub textures: AHashMap<AssetResourceID, Box<dyn Texture>>,
}

impl AssetServer {
    /// Registers a mesh into the Asset Server.
    pub fn register_mesh(&mut self, id: AssetResourceID, mesh: Mesh) {
        self.data
            .write()
            .expect("Unable to acquire write lock")
            .meshes
            .insert(id, Arc::new(mesh));
    }

    pub fn register_texture(
        &mut self,
        id: AssetResourceID,
        texture: Box<dyn Texture>,
    ) {
        self.data
            .write()
            .expect("Unable to acquire write lock")
            .textures
            .insert(id, texture);
    }
}
