use std::f32::consts::PI;

use nalgebra::{Vector2, Vector3};
use shipyard::{UniqueView, UniqueViewMut};

use crate::{
    app::App,
    graphics::{
        components::MeshComponent, gpu::AbstractGpu, mesh::Mesh, vertex::Vertex,
    },
    plugin::Pluggable,
    scene::asset_server::{AssetServer, MeshResourceID},
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
        let mut a_server =
            match app.world.borrow::<UniqueViewMut<AssetServer>>() {
                Ok(s) => s,
                Err(_) => {
                    println!(
                    "Primitives are not configured, AssetServer not configured"
                );
                    return;
                }
            };

        let gpu = match app.world.borrow::<UniqueView<AbstractGpu>>() {
            Ok(s) => s,
            Err(_) => {
                println!("Unable to find gpu abstraction");
                return;
            }
        };

        configure_pentagon_primitive(&gpu, &mut a_server);
        configure_cube_primitive(&gpu, &mut a_server);
        configure_sphere_primitive(&gpu, &mut a_server);
        configure_plane_primitive(&gpu, &mut a_server);
        configure_cone_primitive(&gpu, &mut a_server);
    }
}

// PENTAGON
pub const PENTAGON_PRIMITIVE_ID: &str = "PENTAGON_PRIMITIVE_MESH";

const PENTAGON_VERTICES: &[Vertex] = &[
    Vertex {
        pos: [-0.0868241, 0.49240386, 0.0],
        col: [0.5, 0.0, 0.5],
    },
    Vertex {
        pos: [-0.49513406, 0.06958647, 0.0],
        col: [0.5, 0.0, 0.5],
    },
    Vertex {
        pos: [-0.21918549, -0.44939706, 0.0],
        col: [0.5, 0.0, 0.5],
    },
    Vertex {
        pos: [0.35966998, -0.3473291, 0.0],
        col: [0.5, 0.0, 0.5],
    },
    Vertex {
        pos: [0.44147372, 0.2347359, 0.0],
        col: [0.5, 0.0, 0.5],
    },
];

const PENTAGON_INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

fn configure_pentagon_primitive(gpu: &AbstractGpu, a_server: &mut AssetServer) {
    let v_buffer = gpu.allocate_vertex_buffer(
        "Pentagon primitive vertices",
        bytemuck::cast_slice(PENTAGON_VERTICES),
    );

    let i_buffer = gpu.allocate_index_buffer(
        "Pentagon primitive indices",
        bytemuck::cast_slice(PENTAGON_INDICES),
    );

    let mesh = Mesh::new(v_buffer, i_buffer, PENTAGON_INDICES.len() as u32);

    a_server.register_mesh(PENTAGON_PRIMITIVE_ID.to_owned(), mesh);
}

// CUBE
pub const CUBE_PRIMITIVE_ID: &str = "CUBE_PRIMITIVE_MESH";

pub fn cube_mesh_resource() -> MeshResourceID {
    MeshResourceID(CUBE_PRIMITIVE_ID.to_owned())
}

pub fn cube_mesh_component() -> MeshComponent {
    MeshComponent(cube_mesh_resource())
}

const CUBE_VERTICES: &[Vertex] = &[
    Vertex {
        pos: [-1.0, -1.0, 1.0],
        col: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [1.0, -1.0, 1.0],
        col: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0, 1.0],
        col: [1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-1.0, 1.0, 1.0],
        col: [0.0, 1.0, 1.0],
    },
    // Bottom face.
    Vertex {
        pos: [-1.0, 1.0, -1.0],
        col: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0, -1.0],
        col: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [1.0, -1.0, -1.0],
        col: [0.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-1.0, -1.0, -1.0],
        col: [1.0, 1.0, 1.0],
    },
    // Right face.
    Vertex {
        pos: [1.0, -1.0, -1.0],
        col: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0, -1.0],
        col: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0, 1.0],
        col: [1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [1.0, -1.0, 1.0],
        col: [0.0, 1.0, 1.0],
    },
    // Left face.
    Vertex {
        pos: [-1.0, -1.0, 1.0],
        col: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-1.0, 1.0, 1.0],
        col: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-1.0, 1.0, -1.0],
        col: [0.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-1.0, -1.0, -1.0],
        col: [1.0, 1.0, 1.0],
    },
    // Front face.
    Vertex {
        pos: [1.0, 1.0, -1.0],
        col: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-1.0, 1.0, -1.0],
        col: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-1.0, 1.0, 1.0],
        col: [0.0, 1.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0, 1.0],
        col: [1.0, 1.0, 1.0],
    },
    // Back face.
    Vertex {
        pos: [1.0, -1.0, 1.0],
        col: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-1.0, -1.0, 1.0],
        col: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-1.0, -1.0, -1.0],
        col: [1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [1.0, -1.0, -1.0],
        col: [0.0, 1.0, 1.0],
    },
];

const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 10, 11, 8, // right
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // front
    20, 21, 22, 22, 23, 20, // back
];

fn configure_cube_primitive(gpu: &AbstractGpu, a_server: &mut AssetServer) {
    let v_buffer = gpu.allocate_vertex_buffer(
        "Cube primitive vertices",
        bytemuck::cast_slice(CUBE_VERTICES),
    );

    let i_buffer = gpu.allocate_index_buffer(
        "Cube primitive indices",
        bytemuck::cast_slice(CUBE_INDICES),
    );

    let mesh = Mesh::new(v_buffer, i_buffer, CUBE_INDICES.len() as u32);

    a_server.register_mesh(CUBE_PRIMITIVE_ID.to_owned(), mesh);
}

// SPHERE
pub const SPHERE_PRIMITIVE_ID: &str = "SPHERE_PRIMITIVE_MESH";
const SPHERE_RESOLUTION: usize = 10;

pub fn sphere_mesh_resource() -> MeshResourceID {
    MeshResourceID(SPHERE_PRIMITIVE_ID.to_owned())
}

pub fn sphere_mesh_component() -> MeshComponent {
    MeshComponent(sphere_mesh_resource())
}

fn configure_sphere_primitive(gpu: &AbstractGpu, a_server: &mut AssetServer) {
    let (vertices, indices): (Vec<Vec<Vertex>>, Vec<Vec<u16>>) = [
        Vector3::y(),        // Top
        Vector3::y() * -1.0, // Bottom
        Vector3::x() * -1.0, // Left
        Vector3::x(),        // Right
        Vector3::z() * -1.0, // Front
        Vector3::z(),        // Back
    ]
    .iter()
    .map(|d| face(d, SPHERE_RESOLUTION))
    .unzip();

    let vertices = vertices.into_iter().flatten().collect::<Vec<_>>();
    let indices = indices
        .iter()
        .enumerate()
        .flat_map(|(location, data)| {
            data.iter().map(move |index| {
                index + location as u16 * SPHERE_RESOLUTION.pow(2) as u16
            })
        })
        .collect::<Vec<_>>();

    let v_buffer = gpu.allocate_vertex_buffer(
        "Sphere primitive vertices",
        bytemuck::cast_slice(&vertices),
    );

    let i_buffer = gpu.allocate_index_buffer(
        "Sphere primitive indices",
        bytemuck::cast_slice(&indices),
    );

    let mesh = Mesh::new(v_buffer, i_buffer, indices.len() as u32);

    a_server.register_mesh(SPHERE_PRIMITIVE_ID.to_owned(), mesh);
}

fn face(dir: &Vector3<f32>, resolution: usize) -> (Vec<Vertex>, Vec<u16>) {
    // Square grid.
    let mut vertices = vec![
        Vertex {
            pos: [0.0, 0.0, 0.0],
            col: [0.0, 0.0, 0.0]
        };
        resolution.pow(2)
    ];
    let mut indices = vec![0; (resolution - 1).pow(2) * 2 * 3];

    let a_axis = dir.yzx();
    let b_axis = dir.cross(&a_axis);

    let mut i = 0;
    let mut j = 0;

    for y in 0..resolution {
        for x in 0..resolution {
            let position = Vector2::new(
                x as f32 / (resolution - 1) as f32,
                y as f32 / (resolution - 1) as f32,
            );
            let position3 = dir
                + (position.x - 0.5) * 2.0 * a_axis
                + (position.y - 0.5) * 2.0 * b_axis;

            let position3 = position3.normalize();

            vertices[i] = Vertex {
                pos: [position3.x, position3.y, position3.z],
                col: [position3.x, position3.y, position3.z],
            };

            if x < resolution - 1 && y < resolution - 1 {
                // Triangles
                indices[j] = i as u16;
                indices[j + 1] = i as u16 + 1 + resolution as u16;
                indices[j + 2] = i as u16 + resolution as u16;

                indices[j + 3] = i as u16;
                indices[j + 4] = i as u16 + 1;
                indices[j + 5] = i as u16 + 1 + resolution as u16;

                j += 6;
            }

            i += 1;
        }
    }

    (vertices, indices)
}

// PLANE
pub const PLANE_PRIMITIVE_ID: &str = "PLANE_PRIMITIVE_MESH";

pub fn plane_mesh_resource() -> MeshResourceID {
    MeshResourceID(PLANE_PRIMITIVE_ID.to_owned())
}

pub fn plane_mesh_component() -> MeshComponent {
    MeshComponent(plane_mesh_resource())
}

const PLANE_VERTICES: &[Vertex] = &[
    Vertex {
        pos: [-1.0, 0.0, 1.0],
        col: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [1.0, 0.0, 1.0],
        col: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [1.0, 0.0, -1.0],
        col: [1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-1.0, 0.0, -1.0],
        col: [0.0, 1.0, 1.0],
    },
];

const PLANE_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

fn configure_plane_primitive(gpu: &AbstractGpu, a_server: &mut AssetServer) {
    let v_buffer = gpu.allocate_vertex_buffer(
        "Cube primitive vertices",
        bytemuck::cast_slice(PLANE_VERTICES),
    );

    let i_buffer = gpu.allocate_index_buffer(
        "Cube primitive indices",
        bytemuck::cast_slice(PLANE_INDICES),
    );

    let mesh = Mesh::new(v_buffer, i_buffer, PLANE_INDICES.len() as u32);

    a_server.register_mesh(PLANE_PRIMITIVE_ID.to_owned(), mesh);
}

// CONE
pub const CONE_PRIMITIVE_ID: &str = "CYLINDER_PRIMITIVE_MESH";

pub fn cone_mesh_resource() -> MeshResourceID {
    MeshResourceID(CONE_PRIMITIVE_ID.to_owned())
}

pub fn cone_mesh_component() -> MeshComponent {
    MeshComponent(cone_mesh_resource())
}

fn configure_cone_primitive(gpu: &AbstractGpu, a_server: &mut AssetServer) {
    let (vertices, indices) = generate_cone_mesh(23);

    let v_buffer = gpu.allocate_vertex_buffer(
        "Cube primitive vertices",
        bytemuck::cast_slice(&vertices),
    );

    let i_buffer = gpu.allocate_index_buffer(
        "Cube primitive indices",
        bytemuck::cast_slice(&indices),
    );

    let mesh = Mesh::new(v_buffer, i_buffer, indices.len() as u32);
    a_server.register_mesh(CONE_PRIMITIVE_ID.to_owned(), mesh)
}

fn generate_cone_mesh(resolution: usize) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = vec![
        Vertex {
            pos: [0.0, 1.0, 0.0],
            col: [0.0, 0.0, 0.0]
        };
        resolution + 1
    ];

    let mut index_cout = 0;
    let mut indices: Vec<u16> = vec![0; resolution * 3 + (resolution - 2) * 3];

    let angle = (360.0 as f32).to_radians() / resolution as f32;

    for i in 1..resolution + 1 {
        let local_angle = (i - 1) as f32 * angle;
        let x = local_angle.cos();
        let y = local_angle.sin();

        vertices[i] = Vertex {
            pos: [x, -1.0, y],
            col: [x, 0.0, y],
        };

        indices[index_cout] = 0;
        if i + 1 < resolution + 1 {
            indices[index_cout + 1] = i as u16 + 1;
        } else {
            indices[index_cout + 1] = 1;
        }
        indices[index_cout + 2] = i as u16;

        index_cout += 3;
    }

    for i in 2..vertices.len() - 1 {
        indices[index_cout] = i as u16;

        if i + 1 < resolution + 1 {
            indices[index_cout + 1] = i as u16 + 1;
        } else {
            indices[index_cout + 1] = 1;
        }

        indices[index_cout + 2] = 1;

        index_cout += 3;
    }

    (vertices, indices)
}
