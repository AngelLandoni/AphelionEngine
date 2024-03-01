use shipyard::{Unique, UniqueView, UniqueViewMut, World};
use wgpu::{
    BindGroup, BindGroupLayout, BlendComponent, ColorTargetState, ColorWrites,
    ComputePipeline, FragmentState, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, TextureFormat, TextureUsages, VertexState,
};

use crate::{
    graphics::gpu::AbstractGpu,
    scene::asset_server::AssetServer,
    wgpu_graphics::{buffer::WGPUTexture, gpu::Gpu},
};

// TODO(Angel): Move this out wgpu graphics.
#[derive(Unique)]
pub struct SkyUpdater {
    texture_id: String,
}

#[derive(Unique)]
pub(crate) struct SkyPipeline {
    pub(crate) pipeline: RenderPipeline,

    pub(crate) texture_format: TextureFormat,
    pub(crate) equirect_layout: BindGroupLayout,
    pub(crate) equirectangular_conversion_pipeline: ComputePipeline,

    pub(crate) environment_layout: BindGroupLayout,
    /// Represents the bind group, which is optional because it requires
    /// initializing the scene before creating the bind group.
    pub(crate) environment_bind_group: BindGroup,

    /// Contains a reference to the cube texture which holds the unwraped
    /// equirectangular texure.
    pub(crate) cube_map_texture: WGPUTexture,
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

        // Allocate a new cube map of the size of the
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Sky cubemap texture"),
            size: wgpu::Extent3d {
                width: 1080,
                height: 1080,
                // A cube has 6 sides, so we need 6 layers
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Sky texture view"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sky sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let environment_bind_group =
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("environment_bind_group"),
                layout: &environment_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        SkyPipeline {
            pipeline,
            texture_format,
            equirect_layout,
            equirectangular_conversion_pipeline: equirect_to_cubemap,
            environment_layout,
            environment_bind_group,
            cube_map_texture: WGPUTexture {
                texture,
                view,
                sampler: Some(sampler),
            },
        }
    }
}

/// Takes from the `AssetStorage` the indicated texture
pub(crate) fn configure_sky_pipeline_uniforms(world: &World) {
    let gpu = world.borrow::<UniqueView<AbstractGpu>>().unwrap();
    let asset_server = world.borrow::<UniqueView<AssetServer>>().unwrap();
    let mut sky_pipeline =
        world.borrow::<UniqueViewMut<SkyPipeline>>().unwrap();
    let sky_updater = match world.borrow::<UniqueView<SkyUpdater>>() {
        Ok(s_u) => s_u,
        _ => return,
    };

    // TODO(Angel): When the texture is already processed remove it?.
    let gpu = gpu
        .downcast_ref::<Gpu>()
        .expect("Incorrect GPU type expecting WGPU gpu");

    let asset_lock = &asset_server
        .data
        .read()
        .expect("Unable to take asset store lock on updating sky");

    let texture = match asset_lock.textures.get(sky_updater.texture_id.as_str())
    {
        Some(t) => t,
        _ => return,
    };

    let wgpu_texture = &texture
        .downcast_ref::<WGPUTexture>()
        .expect("Incorrect texture type, expecting WGPUTexture");

    let dst_view = sky_pipeline.cube_map_texture.texture.create_view(
        &wgpu::TextureViewDescriptor {
            label: Some("Sky cubemap copy view"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            // array_layer_count: Some(6),
            ..Default::default()
        },
    );

    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Sky cube map projection bind group"),
        layout: &sky_pipeline.equirect_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &wgpu_texture.view,
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&dst_view),
            },
        ],
    });

    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Cube map texture projection"),
        timestamp_writes: None,
    });

    let num_workgroups = (1080 + 15) / 16;
    pass.set_pipeline(&sky_pipeline.equirectangular_conversion_pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(num_workgroups, num_workgroups, 6);

    drop(pass);

    gpu.queue.submit([encoder.finish()]);
}

/// A free function which removes the `SkyUpdater` component. This is done to
/// avoid texture reallocations every frame, and only when it is just needed.
pub(crate) fn clear_sky_updater(world: &World) {
    world.remove_unique::<SkyUpdater>();
}
