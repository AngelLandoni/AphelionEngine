use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use ahash::AHashMap;
use log::warn;
use nalgebra::Matrix4;
use shipyard::{
    EntitiesView, EntityId, Get, IntoIter, UniqueView, UniqueViewMut, View,
};

use crate::{
    graphics::{
        components::MeshComponent, gpu::AbstractGpu, mesh::Mesh, BindGroup,
        BufferUsage, Texture, UniformBuffer, VertexBuffer,
    },
    scene::{
        assets::{asset_server::AssetServer, AssetResourceID},
        camera::Camera,
        components::Transform,
        hierarchy::Hierarchy,
        projection::Projection,
        scene::SceneTarget,
        scene_state::SceneState,
    },
    wgpu_graphics::passes::forward_pass::{
        ForwardRender, INTERNAL_MAIN_SCENE_ID,
    },
};

use super::material::{Material, MaterialComponent};

/// Describes how many entities which a specific mesh and material
/// can be displayed at the same time.
const MAX_NUMBER_OF_INSANCES_PER_MESH_MATERIAL: u64 = 2000;

/// Represents a combination of a mesh with a material, used to
/// identify the buffer information to be rendered.
#[derive(PartialEq, Eq, Clone)]
pub struct ForwardModelID {
    mesh_id: AssetResourceID,
    material_id: Option<AssetResourceID>,
}

impl Hash for ForwardModelID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.mesh_id.hash(state);
        self.material_id.hash(state);
    }
}

/// A model to be rendered X number of times.
pub(crate) struct ForwardModel {
    /// A reference to the mesh data.
    mesh: Arc<Mesh>,
    /// A refernece to the material data.
    material: Option<Material>,
    /// Contains the chunk of memory on the GPU designated to
    /// the transformations.
    transforms_buffer: Box<dyn VertexBuffer>,
    /// Contains the number of instances to be rendered.
    number_of_instances: Mutex<u64>,
}

unsafe impl Send for ForwardModel {}
unsafe impl Sync for ForwardModel {}

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
        AHashMap<AssetResourceID, (Box<dyn VertexBuffer>, u64)>,

    /// Contains the models to be rendered.
    pub(crate) forward_models: AHashMap<ForwardModelID, ForwardModel>,

    /// Contains the `Texture where the color will be rendered
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
    let mut scene_raw_transforms: AHashMap<AssetResourceID, Vec<u8>> =
        AHashMap::new();

    // Allocate a transform buffer for the mesh if it does not exists.
    for ent in meshes.iter() {
        let id = ent.0.clone();

        scene.mesh_transform_buffers.entry(id).or_insert_with(|| {
            let buffer = gpu.allocate_aligned_zero_vertex_buffer(
                &format!("Mesh({}) transform", ent.0),
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

    scene
        .mesh_transform_buffers
        .iter_mut()
        .filter(|mesh| !scene_raw_transforms.contains_key(mesh.0))
        .for_each(|(_, buffer)| {
            buffer.1 = 0;
        });
}

/// Syncs the buffers and uniforms for all the models in the forward rendering pipeline.
///
/// This system synchronizes the buffers and uniforms for all the models in the forward
/// rendering pipeline across all scenes. It iterates over the main scene and its sub-scenes,
/// allocating transform buffers for meshes and materials that don't already have them.
///
/// # Arguments
///
/// * `gpu` - A unique view of the abstract GPU instance used for synchronization.
/// * `scenes` - A mutable unique view of the scene state containing the main scene and its sub-scenes.
/// * `asset_server` - A unique view of the asset server containing information about meshes and materials.
/// * `entities` - The view of entities in the ECS world.
/// * `forward_renderer` - The view of entities containing the `ForwardRender` component.
/// * `transforms` - The view of transform components for entities.
/// * `meshes` - The view of mesh components for entities.
/// * `materials` - The view of material components for entities.
///
/// # Remarks
///
/// This system iterates over each scene, including the main scene and its sub-scenes.
/// For each scene, it checks entities with mesh and material components and allocates
/// transform buffers if they don't already exist. It also allocates transform buffers
/// for entities with only mesh components and no material components.
///
/// TODO: Do we really want to allocate mesh material data when the sync happens?
pub(crate) fn sync_forward_models_memory_for_all_scenes_system(
    gpu: UniqueView<AbstractGpu>,
    mut scenes: UniqueViewMut<SceneState>,
    asset_server: UniqueView<AssetServer>,
    entities: EntitiesView,
    forward_renderer: View<ForwardRender>,
    transforms: View<Transform>,
    meshes: View<MeshComponent>,
    materials: View<MaterialComponent>,
) {
    let main_scene_temp_id = INTERNAL_MAIN_SCENE_ID.to_owned();

    // To avoid getting a compiler error due to having two mutalbe refs
    // to a single var, we need to extract it field by field.
    let scenes_view_mut = scenes.as_view_mut();

    // Chain the subscenes along side the main scene to make the code
    // easier and cleaner.
    let scenes = scenes_view_mut
        .sub_scenes
        .iter_mut()
        .chain(std::iter::once((&main_scene_temp_id, scenes_view_mut.main)));

    for (id, scene) in scenes {
        // Allocate transform buffer if the mesh + material does not exist.

        // If the entity has mesh and material.
        (&meshes, &materials)
            .iter()
            .for_each(|(mesh_id, material_id)| {
                allocate_transform_buffer_if_it_does_not_exist(
                    &gpu,
                    scene,
                    &mesh_id,
                    Some(&material_id),
                    &asset_server,
                );
            });

        // If the entity hash only mesh.
        entities
            .iter()
            .filter(|e| meshes.contains(*e) && !materials.contains(*e))
            .filter_map(|e| meshes.get(e).ok())
            .for_each(|mesh_id| {
                allocate_transform_buffer_if_it_does_not_exist(
                    &gpu,
                    scene,
                    &mesh_id,
                    None,
                    &asset_server,
                );
            });
    }
}

/// Allocates a transform buffer if it does not exist for a given mesh and material.
///
/// # Arguments
///
/// * `gpu` - The GPU instance used for allocation.
/// * `scene` - A mutable reference to the scene.
/// * `mesh_id` - The resource ID of the mesh.
/// * `material_id` - The resource ID of the material.
/// * `asset_server` - The asset server instance used to retrieve meshes and materials.
///
/// # Remarks
///
/// This function will allocate a transform buffer on the GPU if a forward model for
/// the given mesh and material does not already exist in the scene. The transform
/// buffer is allocated with a name derived from the mesh and material IDs. If the
/// allocation fails or if the mesh does not exist, a warning will be  logged and
/// the function will return without modifying the scene.
///
/// TODO: Replace mutex with atomic.
fn allocate_transform_buffer_if_it_does_not_exist(
    gpu: &AbstractGpu,
    scene: &mut Scene,
    mesh_id: &AssetResourceID,
    material_id: Option<&AssetResourceID>,
    asset_server: &AssetServer,
) {
    let id = ForwardModelID {
        mesh_id: mesh_id.clone(),
        material_id: material_id.map(|material_id| material_id.clone()),
    };

    // Avoid allocation if the model is already represented on GPU memory.
    if scene.forward_models.contains_key(&id) {
        return;
    }

    let mesh = match asset_server.get_mesh(&mesh_id) {
        Some(mesh) => mesh,
        None => {
            warn!("Failed to allocate transform buffer for {} mesh does not exist", mesh_id);
            return;
        }
    };

    let material = {
        if let Some(material_id) = material_id {
            asset_server.get_material(&material_id)
        } else {
            None
        }
    };

    let transforms_buffer = gpu.allocate_aligned_zero_vertex_buffer(
        format!(
            "Forward render: Mesh: {} - Material: {}",
            mesh_id,
            material_id.unwrap_or(&"".to_owned())
        )
        .as_str(),
        Transform::raw_size() * MAX_NUMBER_OF_INSANCES_PER_MESH_MATERIAL,
        BufferUsage::COPY_DST,
    );

    let model = ForwardModel {
        mesh,
        material,
        transforms_buffer,
        number_of_instances: Mutex::new(0),
    };

    scene.forward_models.insert(id, model);
}

/// Synchronizes the transforms hierarchy for all scenes.
///
/// This system syncs the transform hierarchy for all scenes, updating the GPU buffers
/// with the necessary transformation data.
///
/// # Parameters
///
/// - `gpu`: Unique view of the abstract GPU, used for writing to GPU buffers.
/// - `scenes`: Unique view of the scene state, containing main and sub scenes.
/// - `entities`: Reference to the entities view, providing access to entity data.
/// - `meshes`: Reference to the mesh components view, containing mesh data for entities.
/// - `materials`: Reference to the material components view, containing material data for entities.
/// - `transforms`: Reference to the transform components view, containing transformation data for entities.
/// - `hierarchy`: Reference to the hierarchy components view, containing parent-child relationship data for entities.
/// - `scene_targets`: Reference to the scene target components view, specifying the scene each entity belongs to.
pub(crate) fn sync_transforms_hierarchy_system(
    gpu: UniqueView<AbstractGpu>,
    mut scenes: UniqueViewMut<SceneState>,
    entities: EntitiesView,
    meshes: View<MeshComponent>,
    materials: View<MaterialComponent>,
    transforms: View<Transform>,
    hierarchy: View<Hierarchy>,
    scene_targets: View<SceneTarget>,
) {
    // Sort entities by hierarchy level to ensure proper parent-child
    // transformation order.
    let mut sorted_entities = entities.iter().collect::<Vec<EntityId>>();
    sorted_entities.sort_by(|a, b| {
        // Get the hierarchy level for entity 'a'.
        let a_level = match hierarchy.get(*a) {
            Ok(level) => level,
            // Move entities without hierarchy to the end.
            Err(_) => return Ordering::Greater,
        };

        // Get the hierarchy level for entity 'b'.
        let b_level = match hierarchy.get(*b) {
            Ok(level) => level,
            // Move entities without hierarchy to the front.
            Err(_) => return Ordering::Less,
        };

        // Compare the hierarchy levels of 'a' and 'b'.
        a_level.level.cmp(&b_level.level)
    });

    let main_scene_temp_id = INTERNAL_MAIN_SCENE_ID.to_owned();

    // To avoid getting a compiler error due to having two mutalbe refs
    // to a single var, we need to extract it field by field.
    let scenes_view_mut = scenes.as_view_mut();

    // Chain the subscenes along side the main scene to make the code
    // easier and cleaner.
    let scenes = scenes_view_mut
        .sub_scenes
        .iter_mut()
        .chain(std::iter::once((&main_scene_temp_id, scenes_view_mut.main)));

    scenes.for_each(|(scene_id, scene)| {
        let mut scene_raw_transforms: AHashMap<ForwardModelID, Vec<u8>> =
            AHashMap::new();

        let mut accum_transforms: AHashMap<&EntityId, Matrix4<f32>> =
            AHashMap::new();

        sorted_entities
            .iter()
            // Discart entity if it does not have a mesh, no render needed.
            .filter_map(|entity| Some((entity, meshes.get(*entity).ok()?)))
            // Get the material if the entity as one.
            .map(|(entity, mesh)| (entity, mesh, materials.get(*entity).ok()))
            // Discart entity if it does not have a transform.
            .filter_map(|(entity, mesh_id, material_id)| {
                Some((
                    entity,
                    mesh_id,
                    material_id,
                    transforms.get(*entity).ok()?,
                ))
            })
            // Assing entity to main if it does not have any associated scene
            // target.
            .filter_map(|(entity, mesh_id, material_id, transform)| {
                Some((
                    entity,
                    mesh_id,
                    material_id,
                    transform,
                    scene_targets.get(*entity).unwrap_or(&SceneTarget::Main),
                ))
            })
            // Discart the entity if it does not belong to the current scene.
            .filter(|(_, _, _, _, scene_target)| {
                match scene_target {
                    // If the entity is assiged to the main scene, we must
                    // check that we are iterating over the main scene.
                    SceneTarget::Main => {
                        *scene_id == INTERNAL_MAIN_SCENE_ID.to_owned()
                    }
                    // If the entity is assiged to a sub scene we must
                    // make sure that the scene is the correct one.
                    SceneTarget::SubScene(sub_scene) => sub_scene == scene_id,
                }
            })
            .for_each(|(entity, mesh_id, material_id, transform, _)| {
                scene_raw_transforms
                    // Find by mesh and material at the same time.
                    .entry(ForwardModelID {
                        mesh_id: mesh_id.0.clone(),
                        material_id: material_id.map(|id| id.0.clone()),
                    })
                    .and_modify(|raw_transforms| {
                        if let Ok(hierarchy) = hierarchy.get(*entity) {
                            // Get parent transform matrix.
                            if let Some(parent_id) = hierarchy.parent {
                                if let Some(parent_transform) =
                                    accum_transforms.get(&parent_id)
                                {
                                    let carry_transform = parent_transform
                                        * transform.as_matrix();
                                    // If it does not have hierarchy apply normal transform.
                                    let data: [[f32; 4]; 4] =
                                        carry_transform.into();
                                    raw_transforms.extend_from_slice(
                                        bytemuck::cast_slice(&data),
                                    );
                                    accum_transforms
                                        .insert(entity, carry_transform);
                                    return;
                                }
                                // If the parent is not there something wrong happen
                                // as the entities are sorted by level.
                            }
                            // Insert the entity in the accumulator most likely it has
                            // children.
                            accum_transforms
                                .insert(entity, transform.as_matrix());
                        }

                        // If it does not have hierarchy apply normal transform.
                        // There is not need to insert the entity in the trasnformation
                        // accumulation because if the code reach this point it means
                        // that it is a free entity. As the entities are sorted the
                        // parent must be in the accum.
                        let data = transform.as_matrix_array();
                        let a: &[u8] = bytemuck::cast_slice(&data);
                        raw_transforms.extend_from_slice(a);
                    })
                    .or_insert_with(|| {
                        let mut vec = Vec::new();
                        let data = transform.as_matrix_array();
                        let a: &[u8] = bytemuck::cast_slice(&data);
                        vec.extend_from_slice(a);

                        accum_transforms.insert(entity, transform.as_matrix());

                        vec
                    });
            });

        // Write transform to GPU data.
        for (id, transforms_buffer) in scene_raw_transforms.iter() {
            scene.forward_models.entry(id.clone()).and_modify(
                |forward_model| {
                    // Write into the GPU buffer.
                    gpu.write_vertex_buffer(
                        &forward_model.transforms_buffer,
                        0,
                        transforms_buffer,
                    );
                    // Set the counter.
                    *forward_model
                        .number_of_instances
                        .lock()
                        .expect("Unable to get lock") =
                        transforms_buffer.len() as u64 / Transform::raw_size();
                },
            );
        }

        // Clean old transforms.
        scene
            .forward_models
            .iter_mut()
            .filter(|(forward_model_id, _)| {
                !scene_raw_transforms.contains_key(forward_model_id)
            })
            .for_each(|(_, transforms_buffer)| {
                *transforms_buffer
                    .number_of_instances
                    .lock()
                    .expect("Unable to acquire lock") = 0;
            })
    });
}
