use shipyard::{Unique, UniqueView, UniqueViewMut};
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    ShaderStages,
};

use crate::{graphics::gpu::AbstractGpu, scene::scene_state::SceneState};

use super::{
    buffer::{WGPUBindGroup, WgpuUniformBuffer},
    gpu::Gpu,
};

pub(crate) mod dynamic_mesh_pipeline;
pub(crate) mod frame_composition_pipeline;
pub(crate) mod infinite_grid_pipeline;
pub(crate) mod sky_pipeline;

#[derive(Unique)]
pub(crate) struct GlobalBindGroupLayouts {
    pub(crate) camera: BindGroupLayout,
}

/// Creates and returns a commond camera bind group layout.
pub(crate) fn create_camera_bind_group_layout(gpu: &Gpu) -> BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera bind group"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
}

/// Setups the uniforms of all the scenes.
pub(crate) fn setup_scenes_uniforms_system(
    gpu: UniqueView<AbstractGpu>,
    mut s_state: UniqueViewMut<SceneState>,
    global_bind_group_layouts: UniqueView<GlobalBindGroupLayouts>,
) {
    let gpu = gpu
        .downcast_ref::<Gpu>()
        .expect("Incorrect GPU type expecting WGPU gpu");

    // Main scene.
    let camera_buffer = s_state
        .main
        .camera_buffer
        .downcast_ref::<WgpuUniformBuffer>()
        .expect("Incorrect uniform buffer type");

    s_state.main.camera_bind_group = Some(Box::new(WGPUBindGroup(
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_bind_group_layouts.camera,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.0.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        }),
    )));

    // Sub scenes.
    for (_id, scene) in &mut s_state.sub_scenes {
        let camera_buffer = scene
            .camera_buffer
            .downcast_ref::<WgpuUniformBuffer>()
            .expect("Incorrect uniform buffer type");

        scene.camera_bind_group = Some(Box::new(WGPUBindGroup(
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &global_bind_group_layouts.camera,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.0.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            }),
        )));
    }
}
