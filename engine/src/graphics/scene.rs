use std::cmp::Ordering;

use ahash::AHashMap;
use nalgebra::{Matrix4};
use shipyard::{
    EntitiesView, EntityId, Get, IntoIter, UniqueView, UniqueViewMut, View,
};

use crate::{
    graphics::UniformBuffer,
    scene::{
        asset_server::{MeshResourceID},
        camera::Camera,
        components::Transform,
        hierarchy::Hierarchy,
        projection::Projection,
        scene::SceneTarget,
        scene_state::SceneState,
    },
};

use super::{
    components::MeshComponent, gpu::AbstractGpu, BindGroup, BufferUsage,
    Texture, VertexBuffer,
};

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
    pub target_texture: Box<dyn Texture>,
    /// Contains the depth `Texture`.
    pub(crate) depth_texture: Box<dyn Texture>,

    /// Contains the bindgroup used to bind the camera information.
    pub(crate) camera_bind_group: Option<Box<dyn BindGroup>>,

    pub(crate) should_sync_resolution_to_window: bool,

    /// Containst the cube texture used to draw the sky.
    // TODO(Angel): Add to the `SceneDescriptor` a property to disable
    // sky, and make this optional.
    pub(crate) sky_texture: Option<Box<dyn Texture>>,
    /// Contains the sky env bind group.
    pub(crate) sky_env_bind_group: Option<Box<dyn BindGroup>>,
}

pub(crate) fn sync_main_scene_dynamic_entities_transform(
    gpu: UniqueView<AbstractGpu>,
    entities: EntitiesView,
    transforms: View<Transform>,
    scene_targets: View<SceneTarget>,
    meshes: View<MeshComponent>,
    mut scenes: UniqueViewMut<SceneState>,
    hierarchy: View<Hierarchy>,
) {
    // Main scene.
    sync_scene(
        &mut scenes.main,
        None,
        &gpu,
        &entities,
        &transforms,
        &meshes,
        &scene_targets,
        &hierarchy,
    );
    // Sub scenes.
    for (id, scene) in &mut scenes.sub_scenes {
        sync_scene(
            scene,
            Some(id),
            &gpu,
            &entities,
            &transforms,
            &meshes,
            &scene_targets,
            &hierarchy,
        );
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
    hierarchy: &View<Hierarchy>,
) {
    let mut scene_raw_transforms: AHashMap<MeshResourceID, Vec<u8>> =
        AHashMap::new();

    for ent in meshes.iter() {
        let id = ent.0.clone();

        scene.mesh_transform_buffers.entry(id).or_insert_with(|| {
            let buffer = gpu.allocate_aligned_zero_vertex_buffer(
                &format!("Mesh({}) transform", ent.0 .0),
                // TODO(Angel): The size must be configured using the pipeline props.
                200000 * std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                BufferUsage::COPY_DST,
            );
            (buffer, 0)
        });
    }

    // In order to apply hierarchy transformation the entities must be
    // ordered by level, so the children can apply their final parent
    // transformation.
    let mut sorted_vec = entities.iter().collect::<Vec<EntityId>>();
    sorted_vec.sort_by(|a, b| {
        // If `a` does not have hierachy send it to the end of the
        // vector.
        let a_level = match hierarchy.get(*a) {
            Ok(l) => l,
            _ => return Ordering::Greater,
        };
        // Keep it on the left size just to compare it with the next item.
        let b_level = match hierarchy.get(*b) {
            Ok(l) => l,
            _ => return Ordering::Less,
        };

        a_level.level.cmp(&b_level.level)
    });

    let mut accum_transforms: AHashMap<&EntityId, Matrix4<f32>> =
        AHashMap::new();

    for entity_id in sorted_vec.iter() {
        let mesh = match meshes.get(*entity_id) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let transform = match transforms.get(*entity_id) {
            Ok(t) => t,
            Err(_) => continue,
        };

        match scene_targets.get(*entity_id) {
            // If it does not contain the component it must be added to the
            // main scene.
            Ok(SceneTarget::Main) | Err(_) => {
                if scene_id.is_none() {
                    let id = mesh.0.clone();

                    scene_raw_transforms
                        .entry(id)
                        .and_modify(|e| {
                            // Get parent transform matrix.
                            if let Ok(h) = hierarchy.get(*entity_id) {
                                if let Some(parent_id) = h.parent {
                                    if let Some(parent_transform) =
                                        accum_transforms.get(&parent_id)
                                    {
                                        let carry_transform = parent_transform
                                            * transform.as_matrix();
                                        // If it does not have hierarchy apply normal transform.
                                        let data: [[f32; 4]; 4] =
                                            carry_transform.into();
                                        let a: &[u8] =
                                            bytemuck::cast_slice(&data);
                                        e.extend_from_slice(a);
                                        accum_transforms
                                            .insert(entity_id, carry_transform);
                                        return;
                                    }
                                    // If the parent is not there something wrong happen
                                    // as the entities are sorted by level.
                                }
                            }

                            // If it does not have hierarchy apply normal transform.
                            // There is not need to insert the entity in the trasnformation
                            // accumulation because if the code reach this point it means
                            // that it is a free entity. As the entities are sorted the
                            // parent must be in the accum.
                            let data = transform.as_matrix_array();
                            let a: &[u8] = bytemuck::cast_slice(&data);
                            e.extend_from_slice(a);

                            // Insert the entity in the accumulator most likely it has
                            // children.
                            accum_transforms
                                .insert(entity_id, transform.as_matrix());
                        })
                        .or_insert_with(|| {
                            let mut vec = Vec::new();
                            let data = transform.as_matrix_array();
                            let a: &[u8] = bytemuck::cast_slice(&data);
                            vec.extend_from_slice(a);

                            accum_transforms
                                .insert(entity_id, transform.as_matrix());

                            vec
                        });
                }
            }

            Ok(SceneTarget::SubScene(s)) => {
                if let Some(scene_id) = scene_id {
                    // If we found an entity which is assiged to the current
                    // scene add the transformation.
                    if *scene_id == *s {
                        let id = mesh.0.clone();

                        scene_raw_transforms
                            .entry(id)
                            .and_modify(|e| {
                                // Get parent transform matrix.
                                if let Ok(h) = hierarchy.get(*entity_id) {
                                    if let Some(parent_id) = h.parent {
                                        if let Some(parent_transform) =
                                            accum_transforms.get(&parent_id)
                                        {
                                            let carry_transform =
                                                parent_transform
                                                    * transform.as_matrix();
                                            // If it does not have hierarchy apply normal transform.
                                            let data: [[f32; 4]; 4] =
                                                carry_transform.into();
                                            let a: &[u8] =
                                                bytemuck::cast_slice(&data);
                                            e.extend_from_slice(a);
                                            accum_transforms.insert(
                                                entity_id,
                                                carry_transform,
                                            );
                                            return;
                                        }
                                        // If the parent is not there something wrong happen
                                        // as the entities are sorted by level.
                                    }
                                    // Insert the entity in the accumulator most likely it has
                                    // children.
                                    accum_transforms.insert(
                                        entity_id,
                                        transform.as_matrix(),
                                    );
                                }

                                // If it does not have hierarchy apply normal transform.
                                // There is not need to insert the entity in the trasnformation
                                // accumulation because if the code reach this point it means
                                // that it is a free entity. As the entities are sorted the
                                // parent must be in the accum.
                                let data = transform.as_matrix_array();
                                let a: &[u8] = bytemuck::cast_slice(&data);
                                e.extend_from_slice(a);
                            })
                            .or_insert_with(|| {
                                let mut vec = Vec::new();
                                let data = transform.as_matrix_array();
                                let a: &[u8] = bytemuck::cast_slice(&data);
                                vec.extend_from_slice(a);

                                accum_transforms
                                    .insert(entity_id, transform.as_matrix());

                                vec
                            });
                    }
                }
            }
        }
    }

    for (m, b) in scene_raw_transforms.iter() {
        scene
            .mesh_transform_buffers
            .entry(m.clone())
            .and_modify(|e| {
                gpu.write_vertex_buffer(&e.0, 0, b);
                e.1 = b.len() as u64 / Transform::raw_size();
            });
    }
}
