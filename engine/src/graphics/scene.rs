use ahash::AHashMap;
use nalgebra::Matrix;
use shipyard::{EntitiesView, Get, IntoIter, UniqueView, UniqueViewMut, View};

use crate::{
    graphics::UniformBuffer,
    scene::{
        asset_server::MeshResourceID, camera::Camera, components::Transform, projection::Projection, scene::SceneTarget, scene_state::SceneState
    },
};

use super::{components::MeshComponent, gpu::AbstractGpu, BufferUsage, Texture, VertexBuffer};

pub struct Scene {
    /// Contains a debug tag.
    pub label: String,
    /// Contains the `Camera` used in the scene.
    pub camera: Camera,
    /// Contains the `Projection` used.
    pub projection: Projection,

    /// Conaints the camera information allocated in the GPU RAM.
    pub(crate) camera_buffer: Box<dyn UniformBuffer>,
    /// Contains the buffer which holds the transform information.
    // TODO(Angel): Set this as u32, WGPU only supports u32 for instancing
    pub(crate) mesh_transform_buffers:
        AHashMap<MeshResourceID, (Box<dyn VertexBuffer>, u64)>,

    /// Contains the `Texture where the color will be rendered
    // TODO(Angel): Add here PBR.
    pub(crate) target_texture: Box<dyn Texture>,
    /// Contains the depth `Texture`.
    pub(crate) depth_texture: Box<dyn Texture>,

    pub(crate) should_sync_resolution_to_window: bool,
}

impl Scene {
    /// Calculates and returns the camera projection matrix.
    pub fn calculate_camera_projection(
        &self,
    ) -> Matrix<
        f32,
        nalgebra::Const<4>,
        nalgebra::Const<4>,
        nalgebra::ArrayStorage<f32, 4, 4>,
    > {
        self.projection.matrix() * self.camera.view_matrix()
    }
}

// TODO(Angel): Add support for multi scene.
pub(crate) fn sync_main_scene_dynamic_entities_transform(
    gpu: UniqueView<AbstractGpu>,
    entities: EntitiesView,
    transforms: View<Transform>,
    scene_targets: View<SceneTarget>,
    meshes: View<MeshComponent>,
    mut scenes: UniqueViewMut<SceneState>,
) {
    let mut main_scene_raw_transforms: AHashMap<MeshResourceID, Vec<u8>> = AHashMap::new();

    let main_scene = &mut scenes.main;

    for ent in meshes.iter() {
        main_scene
            .mesh_transform_buffers
            .entry(**ent)
            .or_insert_with(|| {
                let buffer = gpu.allocate_aligned_zero_vertex_buffer(
                    &format!("Mesh({}) transform", ent.0 .0),
                    // TODO(Angel): The size must be configured using the pipeline props.
                    200000 * std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                    BufferUsage::COPY_DST,
                );
                (buffer, 0)
            });
    }

    for e in entities.iter() {
        let mesh = match meshes.get(e) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let transform = match transforms.get(e) {
            Ok(t) => t,
            Err(_) => continue,
        };

        match scene_targets.get(e) {
            // If it does not contain the component it must be added to the
            // main scene.
            Ok(SceneTarget::Main) | Err(_) => {
                main_scene_raw_transforms
                    .entry(**mesh)
                    .and_modify(|e| {
                        let data = transform.as_matrix_array();
                        let a: &[u8] = bytemuck::cast_slice(&data);
                        e.extend_from_slice(a);
                    })
                    .or_insert_with(|| {
                        let mut vec = Vec::new();
                        let data = transform.as_matrix_array();
                        let a: &[u8] = bytemuck::cast_slice(&data);
                        vec.extend_from_slice(a);
                        vec
                    });
            },

            Ok(SceneTarget::SubScene(s)) => {
                // TODO(Angel): Add support for mutli scene.
            },
        }
    }

    for (m, b) in main_scene_raw_transforms.iter() {
        scenes.main.mesh_transform_buffers.entry(*m).and_modify(|e| {
            gpu.write_vertex_buffer(&e.0, 0, b);
            e.1 = b.len() as u64 / Transform::raw_size();
        });
    }

}

/*fn sync_dynamic_entities_position_system(
    gpu: UniqueView<AbstractGpu>,
    transforms: View<Transform>,
    meshes: View<MeshComponent>,
) {


    pipeline
        .mesh_transform_buffers
        .iter_mut()
        .for_each(|e| e.1 .1 = 0);

    // TODO(Angel): Since we already know the maximum size per mesh, we can
    // pre-allocate memory for each mesh to avoid dynamic reallocation during
    // runtime, which can improve performance by reducing memory fragmentation
    // and allocation overhead.
    let mut raw_transforms: AHashMap<MeshResourceID, Vec<u8>> = AHashMap::new();

    for ent in meshes.iter() {
        pipeline
            .mesh_transform_buffers
            .entry(**ent)
            .or_insert_with(|| {
                // Allocate the buffer.
                let buffer = gpu.allocate_aligned_zero_buffer(
                    &format!("Mesh({}) transform", ent.0 .0),
                    // TODO(Angel): The size must be configured using the pipeline props.
                    200000 * std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                    BufferUsages::VERTEX | BufferUsages::COPY_DST,
                );
                (buffer, 0)
            });
    }

    for (e, t) in (&meshes, &transforms).iter() {
        raw_transforms
            .entry(**e)
            .and_modify(|e| {
                let data = t.as_matrix_array();
                let a: &[u8] = bytemuck::cast_slice(&data);
                e.extend_from_slice(a);
            })
            .or_insert_with(|| {
                let mut vec = Vec::new();
                let data = t.as_matrix_array();
                let a: &[u8] = bytemuck::cast_slice(&data);
                vec.extend_from_slice(a);
                vec
            });
    }

    for (m, b) in raw_transforms.iter() {
        pipeline.mesh_transform_buffers.entry(*m).and_modify(|e| {
            gpu.queue.write_buffer(&e.0, 0, b);
            e.1 = b.len() as u64 / Transform::raw_size();
        });
    }
}
*/
