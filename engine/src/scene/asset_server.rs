use std::{
    collections::HashMap, default, sync::{
        Arc,
        RwLock,
        atomic::{AtomicU64, Ordering}
    }
};

use shipyard::Unique;

use crate::graphics::mesh::Mesh;

type AssetResourceID = u64;

pub struct MeshResourceID(AssetResourceID);

/// Conatins all the assets which a `Scene` can use.
#[derive(Unique)]
pub struct AssetServer {
    data: Arc<AssetServerData>,
}

impl Default for AssetServer {
    fn default() -> Self {
        Self { 
            data: Arc::new(AssetServerData::default()),
        }
    }
}

/// Shipyard requires that `AssetServe` can be used from different threads. The
/// `AssetServerDat` must be protected.
unsafe impl Send for AssetServer {}
unsafe impl Sync for AssetServer {}

struct AssetServerData {
    // Used to store and retrieve the last id used. The only purpose of this is
    // to generate unique ids.
    resouce_id_counter: AtomicU64,
    meshes: RwLock<HashMap<AssetResourceID, Mesh>>,
}

impl Default for AssetServerData {
    fn default() -> Self {
        Self {
            resouce_id_counter: AtomicU64::new(0),
            meshes: RwLock::new(HashMap::new()),
        }
    }
}

impl AssetServer {
    /// Registers a mesh into the Asset Server.
    pub fn register_mesh(&mut self, mesh: Mesh) -> MeshResourceID {
        let id = self.data.resouce_id_counter.fetch_add(1, Ordering::Relaxed);

        self
            .data
            .meshes
            .write()
            .expect("Unable to take mesh lock")
            .insert(id, mesh);

        MeshResourceID(id)
    }
}