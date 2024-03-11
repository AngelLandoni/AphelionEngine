use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use ahash::AHashMap;
use shipyard::Unique;

use crate::graphics::{gpu::AbstractGpu, mesh::Mesh, Texture};

use super::{asset_loader::AssetLoader, AssetResourceID, MeshResourceID};

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
            .get(&mesh.0)
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

    /// Registers a custom mesh into the server.
    pub fn register_mesh_using_path(
        &mut self,
        gpu: &AbstractGpu,
        id: AssetResourceID,
        vertices: &[u8],
        indices: &[u8],
    ) {
        let v_buffer = gpu.allocate_vertex_buffer(
            "Sphere primitive vertices",
            bytemuck::cast_slice(vertices),
        );

        let i_buffer = gpu.allocate_index_buffer(
            "Sphere primitive indices",
            bytemuck::cast_slice(indices),
        );

        self.register_mesh(
            id,
            Mesh::new(v_buffer, i_buffer, indices.len() as u32),
        )
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
