use shipyard::{UniqueView, UniqueViewMut};

use crate::{
    app::App,
    graphics::{
        gpu::AbstractGpu,
        mesh::Mesh,
        vertex::Vertex
    },
    plugin::Pluggable,
    scene::asset_server::{AssetServer, MeshResourceID} 
};

/// Allocates and setups all the default primitives (Triangle, Quad, Cube, Cone,
/// Donut, etc) to be used in the rendering step.
/// 
/// This plugin requires the `AssetServer` therfore the `ScenePlugin` must be
/// configured first.
pub struct PrimitivesPlugin;

impl Pluggable for PrimitivesPlugin {
    fn configure(&self, app: &mut App) {
        // Borrow the server in order to insert all the primitive meshes.
        let mut a_server = match app.world.borrow::<UniqueViewMut<AssetServer>>() {
            Ok(s) => s,
            Err(_) => {
                println!("Primitives are not configured, AssetServer not configured");
                return;
            }
        };

        let gpu = match app
            .world
            .borrow::<UniqueView<AbstractGpu>>() {
                Ok(s) => s,
                Err(_) => {
                    println!("Unable to find gpu abstraction");
                    return;
                } 
            };
        
        configure_pentagon_primitive(&gpu, &mut a_server);
    }
}

/// Conatins the id used to retrieve the information from the `AssetServer`.
const PENTAGON_PRIMITIVE_ID: &str = "PENTAGON_PRIMITIVE_MESH";

const PENTAGON_VERTICES: &[Vertex] = &[
    Vertex { pos: [-0.0868241, 0.49240386, 0.0], col: [0.5, 0.0, 0.5] },
    Vertex { pos: [-0.49513406, 0.06958647, 0.0], col: [0.5, 0.0, 0.5] },
    Vertex { pos: [-0.21918549, -0.44939706, 0.0], col: [0.5, 0.0, 0.5] },
    Vertex { pos: [0.35966998, -0.3473291, 0.0], col: [0.5, 0.0, 0.5] },
    Vertex { pos: [0.44147372, 0.2347359, 0.0], col: [0.5, 0.0, 0.5] },
];

const PENTAGON_INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

pub const PENTAGON_MESH_RESOURCE_ID: MeshResourceID = MeshResourceID(PENTAGON_PRIMITIVE_ID);

fn configure_pentagon_primitive(gpu: &AbstractGpu, a_server: &mut AssetServer) {
    let v_buffer = gpu.allocate_vertex_buffer(
        "Pentagon primitive vertices",
        bytemuck::cast_slice(PENTAGON_VERTICES),
    );

    let i_buffer = gpu.allocate_index_buffer(
        "Pentagon primitive indices",
        bytemuck::cast_slice(PENTAGON_INDICES),
    );

    let mesh = Mesh::new(
        v_buffer,
        i_buffer,
        PENTAGON_INDICES.len() as u32
    );

    a_server.register_mesh(PENTAGON_PRIMITIVE_ID, mesh);
}