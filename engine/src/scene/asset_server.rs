use std::{
    collections::HashMap,
    ops::Deref,
    sync::{
        Arc,
        RwLock,
    }
};

use shipyard::Unique;

use crate::graphics::mesh::Mesh;

type AssetResourceID = &'static str;

pub struct MeshResourceID(pub(crate) AssetResourceID);

impl Deref for MeshResourceID {
    type Target = AssetResourceID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Conatins all the assets which a `Scene` can use.
#[derive(Unique)]
pub struct AssetServer {
    data: Arc<RwLock<AssetServerData>>,
}

impl Default for AssetServer {
    fn default() -> Self {
        Self { 
            data: Arc::new(RwLock::new(AssetServerData::default())),
        }
    }
}

impl AssetServer {
    /// Retrieves a particular `Mesh`.
    pub fn load_mesh(&self, mesh: &MeshResourceID) -> Arc<Mesh> {
        self
            .data
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

struct AssetServerData {
    meshes: HashMap<AssetResourceID, Arc<Mesh>>,
}

impl Default for AssetServerData {
    fn default() -> Self {
        Self {
            meshes: HashMap::new(),
        }
    }
}

impl AssetServer {
    /// Registers a mesh into the Asset Server.
    pub fn register_mesh(
        &mut self, id: AssetResourceID, mesh: Mesh
    ) {
        self
            .data
            .write()
            .expect("Unable to acquire write lock")
            .meshes
            .insert(id, Arc::new(mesh));
    }
}