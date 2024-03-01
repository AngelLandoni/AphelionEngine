use shipyard::{Unique, UniqueView, UniqueViewMut};
use wgpu::{
    BindGroup, BindGroupLayout, BlendComponent, ColorTargetState, ColorWrites,
    ComputePipeline, FragmentState, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, TextureFormat, TextureUsages, VertexState,
};

use crate::{graphics::gpu::AbstractGpu, wgpu_graphics::gpu::Gpu};

#[derive(Unique)]
pub(crate) struct SkyPipeline {
    pub(crate) pipeline: RenderPipeline,

    pub(crate) texture_format: TextureFormat,
    pub(crate) equirect_layout: BindGroupLayout,
    pub(crate) equirectangular_conversion_pipeline: ComputePipeline,

    pub(crate) environment_layout: BindGroupLayout,
    /// Represents the bind group, which is optional because it requires
    /// initializing the scene before creating the bind group.
    pub(crate) environment_bind_group: Option<BindGroup>,
}

impl SkyPipeline {
    pub(crate) fn new(
        gpu: &Gpu,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> SkyPipeline {
        let program =
            gpu.compile_program("sky", include_str!("../shaders/sky.wgsl"));

        let environment_layout = gpu.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("environment_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: false,
                            },
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::NonFiltering,
                        ),
                        count: None,
                    },
                ],
            },
        );

        let layout =
            gpu.device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Sky pipeline layout"),
                    bind_group_layouts: &[
                        camera_bind_group_layout,
                        &environment_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let pipeline =
            gpu.device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("Sky render pipeline"),
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

        let module = gpu.device.create_shader_module(wgpu::include_wgsl!(
            "../shaders/equirectangular.wgsl"
        ));
        let texture_format = wgpu::TextureFormat::Rgba32Float;
        let equirect_layout = gpu.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Equirect layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: false,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: texture_format,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                ],
            },
        );

        let pipeline_layout = gpu.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&equirect_layout],
                push_constant_ranges: &[],
            },
        );

        let equirect_to_cubemap = gpu.device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some("equirect_to_cubemap"),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: "compute_equirect_to_cubemap",
            },
        );

        SkyPipeline {
            pipeline,
            texture_format,
            equirect_layout,
            equirectangular_conversion_pipeline: equirect_to_cubemap,
            environment_layout,
            environment_bind_group: None,
        }
    }
}

pub(crate) fn configure_sky_pipeline_uniforms_system(
    gpu: UniqueView<AbstractGpu>,
    mut sky_pipeline: UniqueViewMut<SkyPipeline>,
) {
    let gpu = gpu
        .downcast_ref::<Gpu>()
        .expect("Incorrect GPU type expecting WGPU gpu");

    /*
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &texture_data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * texture_data.width()),
            rows_per_image: std::num::NonZeroU32::new(texture_data.height()),
        },
        texture_size,
    );
     */
}
