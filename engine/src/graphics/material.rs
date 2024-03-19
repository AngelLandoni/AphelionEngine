use std::ops::Deref;

use ahash::HashMap;
use shipyard::Component;
use wgpu::{BindGroupLayout, RenderPipeline};

use crate::{
    scene::assets::{asset_server::AssetServer, AssetResourceID},
    wgpu_graphics::{
        gpu::Gpu, pipelines::forward_pipeline::create_forward_pipeline,
    },
};

/// Represents a material component associated with an asset resource ID.
///
/// This component stores the ID of the material associated with an entity.
#[derive(Component)]
pub struct MaterialComponent(pub AssetResourceID);

impl MaterialComponent {
    /// Creates a new instance of `MaterialComponent` with the given asset resource ID.
    pub fn new(id: AssetResourceID) -> Self {
        Self(id)
    }
}

impl Deref for MaterialComponent {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents all the possible types of textures associated with a material.
pub enum MaterialTexture {
    /// Diffuse texture.
    Diffuse,
}

impl MaterialTexture {
    /// Returns the texture type as a string.
    pub fn as_string(&self) -> String {
        match self {
            MaterialTexture::Diffuse => "tex_diffuse".to_owned(),
        }
    }
}

/// A convenient type used to describe the storage location of all the textures associated with a material.
type MaterialTextures = HashMap<String, AssetResourceID>;

/// Defines the type of material we are dealing with.
///
/// This is usefull to determine which data we should pass to the
/// shader.
#[derive(Copy, Clone)]
pub enum MaterialKind {
    Debug,
    Untextured,
}

impl MaterialKind {
    fn fragment_shader(&self) -> &'static str {
        match self {
            MaterialKind::Debug => {
                include_str!("shaders/materials/debug.wgsl")
            }
            MaterialKind::Untextured => {
                include_str!("shaders/materials/untextured.wgsl")
            }
        }
    }

    fn bind_group_layouts(&self, gpu: &Gpu) -> Option<BindGroupLayout> {
        match self {
            MaterialKind::Debug => None,
            MaterialKind::Untextured => None,
        }
    }
}

/// Represents a material stored in the GPU.
///
/// This material contains a collection of textures.
#[derive(Clone)]
pub struct Material {
    pub kind: MaterialKind,
    /// Contains all the available textures for the material.
    pub textures: MaterialTextures,
    /// COntains a reference to the pipeline.
    pub(crate) pipeline_id: AssetResourceID,
}

fn generate_material_pipeline(
    kind: MaterialKind,
    gpu: &Gpu,
    camera_bind_group_layout: &BindGroupLayout,
) -> RenderPipeline {
    let fragment_program = gpu
        .compile_program("Untextured fragment shader", kind.fragment_shader());

    create_forward_pipeline(
        gpu,
        camera_bind_group_layout,
        kind.bind_group_layouts(gpu).as_ref(),
        &fragment_program,
    )
}

/// Registers the default material into the system.
pub(crate) fn register_materials(
    gpu: &Gpu,
    asset_server: &mut AssetServer,
    camera_bind_group_layout: &BindGroupLayout,
) {
    // Debug.
    let pipeline_id = "debug_material_pipeline".to_owned();

    asset_server.register_material_pipeline(
        pipeline_id.clone(),
        generate_material_pipeline(
            MaterialKind::Debug,
            gpu,
            camera_bind_group_layout,
        ),
    );

    asset_server.register_material(
        "debug_material".to_string(),
        Material {
            kind: MaterialKind::Debug,
            textures: Default::default(),
            pipeline_id,
        },
    );

    // Untextured.
    let pipeline_id = "untextured_material_pipeline".to_owned();

    asset_server.register_material_pipeline(
        pipeline_id.clone(),
        generate_material_pipeline(
            MaterialKind::Untextured,
            gpu,
            camera_bind_group_layout,
        ),
    );

    asset_server.register_material(
        "untextured_material".to_string(),
        Material {
            kind: MaterialKind::Untextured,
            textures: Default::default(),
            pipeline_id,
        },
    );

    // Others.
}
