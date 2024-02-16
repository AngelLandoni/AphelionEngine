use ahash::AHashMap;
use shipyard::{IntoIter, Unique, UniqueView, UniqueViewMut, View};

use wgpu::{
    vertex_attr_array, BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BlendComponent, Buffer, BufferAddress, BufferUsages, ColorTargetState, ColorWrites, DepthBiasState, DepthStencilState, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StencilState, VertexBufferLayout, VertexState
};

use crate::{
    app::App,
    graphics::{components::MeshComponent, gpu::AbstractGpu, vertex::Vertex},
    scene::{asset_server::MeshResourceID, components::Transform, scene_state::SceneState},
    schedule::Schedule,
    wgpu_graphics::{buffer::WGPUTexture, gpu::{Gpu, DEPTH_TEXTURE_FORMAT}},
};

#[derive(Unique)]
pub struct FrameCompositionPipeline {
    /// Contains a reference to the pipeline.
    pub(crate) pipeline: RenderPipeline,
    pub(crate) texture_bind_group_layout: BindGroupLayout,
    /// Represents the bind group, which is optional because it requires 
    /// initializing the scene before creating the bind group.
    pub(crate) texture_bind_group: Option<BindGroup>,
}

impl FrameCompositionPipeline {
    /// Creates and returns a new `FrameCompositionPipeline`.
    pub(crate) fn new(gpu: &Gpu) -> Self {
        let program = gpu.compile_program(
            "frame_composition",
            include_str!("../shaders/frame_composition.wgsl"),
        );

        let texture_bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("frame_composition_texture_bind_group_layout"),
        });

        let layout = gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Frame composition pipeline layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Frame composition render pipeline"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &program,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &program,
                    entry_point: "fs_main",
                    targets: &[Some(ColorTargetState {
                        format: gpu.surface_config.format,
                        blend: Some(wgpu::BlendState {
                            color: BlendComponent::REPLACE,
                            alpha: BlendComponent::REPLACE,
                        }),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        FrameCompositionPipeline {
            pipeline,
            texture_bind_group_layout,
            texture_bind_group: None,
        }
    }
}

pub(crate) fn setup_frame_composition_pipelines_uniforms_system(
    gpu: UniqueView<AbstractGpu>,
    mut frame_conf: UniqueViewMut<FrameCompositionPipeline>,
    s_state: UniqueView<SceneState>,
) {
    let gpu = gpu.downcast_ref::<Gpu>().expect("Incorrect GPU type expecting WGPU gpu");

    let main_texture = s_state
        .main
        .target_texture
        .downcast_ref::<WGPUTexture>()
        .expect("Incorrect Texture");

    // TODO(Angel): Maybe option is not needed.
    let main_sampler = main_texture.sampler.as_ref().expect("Unable to find sampler");

    let texture_bind_group = gpu.device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &frame_conf.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&main_texture.view),
                },

                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&main_sampler),
                }
            ],
            label: Some("frame_composition_diffuse_bind_group"),
        }
    );

    frame_conf.texture_bind_group = Some(texture_bind_group);
}