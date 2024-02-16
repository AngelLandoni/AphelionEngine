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

pub(crate) fn sync_main_scene_dynamic_entities_transform(
    gpu: UniqueView<AbstractGpu>,
    entities: EntitiesView,
    transforms: View<Transform>,
    scene_targets: View<SceneTarget>,
    meshes: View<MeshComponent>,
    mut scenes: UniqueViewMut<SceneState>,
) {
    // Main scene.
    sync_scene(&mut scenes.main, None, &gpu, &entities, &transforms, &meshes, &scene_targets);
    // Sub scenes.
    for (id, scene) in &mut scenes.sub_scenes {
        sync_scene(scene, Some(id), &gpu, &entities, &transforms, &meshes, &scene_targets);
    }
}


fn sync_scene(
    scene: &mut Scene,
    scene_id: Option<&String>,
    gpu: &UniqueView<AbstractGpu>,
    entities: &EntitiesView,
    transforms: &View<Transform>,
    meshes: &View<MeshComponent>,
    scene_targets: &View<SceneTarget>,
) {
    let mut scene_raw_transforms: AHashMap<MeshResourceID, Vec<u8>> = AHashMap::new();

    for ent in meshes.iter() {
        scene
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
                if scene_id.is_none() {
                    scene_raw_transforms
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
                }
            },

            Ok(SceneTarget::SubScene(s)) => {
                if let Some(scene_id) = scene_id {
                    // If we found an entity which is assiged to the current
                    // scene add the transformation.
                    if *scene_id == *s {
                        scene_raw_transforms
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
                    } 
                }
            },
        }
    }

    for (m, b) in scene_raw_transforms.iter() {
        scene.mesh_transform_buffers.entry(*m).and_modify(|e| {
            gpu.write_vertex_buffer(&e.0, 0, b);
            e.1 = b.len() as u64 / Transform::raw_size();
            println!("Numb instances: {}", e.1);
        });
    }
}