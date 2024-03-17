use std::sync::{Arc, RwLock};

use ahash::AHashMap;
use shipyard::Unique;

use crate::{
    graphics::{gpu::AbstractGpu, material::Material, mesh::Mesh, Texture},
    scene::assets::{asset_loader::AssetLoader, model::Model, AssetResourceID},
    types::Size,
};

/// Contains all the assets which a `Scene` can use.
#[derive(Unique, Default)]
pub struct AssetServer {
    /// Data storage for assets and their loader.
    data: Arc<RwLock<AssetServerData>>,
    /// The loader for assets.
    pub loader: Arc<RwLock<AssetLoader>>,
}

impl AssetServer {
    /// Creates a new `AssetServer`.
    pub fn new() -> Self {
        AssetServer {
            data: Arc::new(RwLock::new(AssetServerData::default())),
            loader: Arc::new(RwLock::new(AssetLoader::default())),
        }
    }

    /// Retrieves a particular `Mesh` by its resource ID.
    ///
    /// # Arguments
    ///
    /// * `mesh` - The resource ID of the mesh to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Mesh` if found, otherwise `None`.
    pub fn load_mesh(&self, mesh: &AssetResourceID) -> Option<Arc<Mesh>> {
        self.data
            .read()
            .expect("Unable to acquire read lock")
            .meshes
            .get(mesh)
            .cloned()
    }

    /// Returns a list of resource IDs for all the currently loaded meshes in the server.
    ///
    /// # Returns
    ///
    /// A `Vec` containing `MeshResourceID` instances for each loaded mesh.
    pub fn meshes(&self) -> Vec<AssetResourceID> {
        self.data
            .read()
            .expect("Unable to acquire read lock")
            .meshes
            .keys()
            .cloned()
            .collect()
    }

    /// Retrieves a mesh by its resource ID.
    ///
    /// # Arguments
    ///
    /// * `mesh_id` - The resource ID of the mesh to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the mesh if found, otherwise `None`.
    pub fn get_mesh(&self, mesh_id: &String) -> Option<Arc<Mesh>> {
        self.data
            .read()
            .expect("Unable to acquire read lock")
            .meshes
            .get(mesh_id)
            .map(|d| d.clone())
    }

    /// Retrieves a material by its resource ID.
    ///
    /// # Arguments
    ///
    /// * `material_id` - The resource ID of the material to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the material if found, otherwise `None`.
    pub fn get_material(
        &self,
        material_id: &String,
    ) -> Option<Arc<dyn Material>> {
        self.data
            .read()
            .expect("Unable to acquire read lock")
            .materials
            .get(material_id)
            .map(|d| d.clone())
    }

    /// Registers a mesh into the Asset Server.
    ///
    /// # Arguments
    ///
    /// * `id` - The resource ID of the mesh.
    /// * `mesh` - The mesh to register.
    pub fn register_mesh(&self, id: AssetResourceID, mesh: Mesh) {
        self.data
            .write()
            .expect("Unable to acquire write lock")
            .meshes
            .insert(id, Arc::new(mesh));
    }

    /// Registers a custom mesh into the server.
    ///
    /// # Arguments
    ///
    /// * `gpu` - The abstract GPU instance.
    /// * `id` - The resource ID of the mesh.
    /// * `vertices` - The vertex data of the mesh.
    /// * `indices` - The index data of the mesh.
    pub fn register_mesh_using_path(
        &self,
        gpu: &AbstractGpu,
        id: AssetResourceID,
        vertices: &[u8],
        indices: &[u8],
    ) {
        let v_buffer =
            gpu.allocate_vertex_buffer("Custom mesh vertices", vertices);
        let i_buffer =
            gpu.allocate_index_buffer("Custom mesh indices", indices);
        let mesh = Mesh::new(v_buffer, i_buffer, indices.len() as u32);
        self.register_mesh(id, mesh);
    }

    /// Registers a texture into the Asset Server.
    ///
    /// # Arguments
    ///
    /// * `id` - The resource ID of the texture.
    /// * `texture` - The texture to register.
    pub fn register_texture(
        &self,
        id: AssetResourceID,
        texture: Arc<dyn Texture>,
    ) {
        self.data
            .write()
            .expect("Unable to acquire write lock")
            .textures
            .insert(id, texture);
    }

    /// Retrieves a texture by its resource ID.
    ///
    /// # Arguments
    ///
    /// * `texture_id` - The resource ID of the texture to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the texture if found, otherwise `None`.
    pub fn get_texture(
        &self,
        texture_id: &AssetResourceID,
    ) -> Option<Arc<dyn Texture>> {
        Some(
            self.data
                .read()
                .expect("Unable to acquire read lock")
                .textures
                .get(texture_id)?
                .clone(),
        )
    }

    /// Retrieves all textures stored in the asset manager.
    ///
    /// # Returns
    ///
    /// A vector containing tuples of resource IDs and references to textures.
    pub fn get_textures(&self) -> Vec<(AssetResourceID, Arc<dyn Texture>)> {
        self.data
            .read()
            .expect("Unable to acquire read lock")
            .textures
            .clone()
            .into_iter()
            .collect()
    }

    /// Extracts textures to be loaded.
    ///
    /// This method retrieves textures from the asset loader, allowing access to the
    /// textures waiting to be loaded. It clears the internal storage of textures
    /// to be loaded.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the texture data to be loaded. Each tuple contains:
    /// - The texture's name.
    /// - The raw texture data.
    /// - The dimensions (size) of the texture.
    pub fn extract_textures_to_load(
        &self,
    ) -> Vec<(String, Vec<u8>, Size<u32>)> {
        // Acquire a write lock on the asset loader to access and modify the textures to load.
        let mut lock =
            self.loader.write().expect("Unable to acquire write lock");

        // Take ownership of the textures to load from the asset loader,
        // leaving it in a clean state with an empty list of textures to load.
        std::mem::take(&mut lock.texture_to_load)
    }

    /// Extracts meshes to be loaded.
    ///
    /// This method retrieves meshes from the asset loader, allowing access to the
    /// meshes waiting to be loaded. It clears the internal storage of meshes
    /// to be loaded.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the mesh data to be loaded. Each tuple contains:
    /// - The mesh's name.
    /// - The model data representing the mesh.
    pub fn extract_meshes_to_load(&self) -> Vec<(String, Model)> {
        // Acquire a write lock on the asset loader to access and modify the meshes to load.
        let mut lock =
            self.loader.write().expect("Unable to acquire write lock");

        // Take ownership of the meshes to load from the asset loader,
        // leaving it in a clean state with an empty list of meshes to load.
        std::mem::take(&mut lock.models_to_load)
    }
}

/// Shipyard requires that `AssetServe` can be used from different threads. The
/// `AssetServerData` must be protected.
unsafe impl Send for AssetServer {}
unsafe impl Sync for AssetServer {}

/// Data structure for storing assets.
#[derive(Default)]
struct AssetServerData {
    /// Contains all the available meshes.
    meshes: AHashMap<AssetResourceID, Arc<Mesh>>,
    /// Contains all the available textures.
    textures: AHashMap<AssetResourceID, Arc<dyn Texture>>,
    /// Contains all the available materials.
    materials: AHashMap<AssetResourceID, Arc<dyn Material>>,
}
