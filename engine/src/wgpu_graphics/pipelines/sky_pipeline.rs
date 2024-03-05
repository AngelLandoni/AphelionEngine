use log::{debug, info};
use shipyard::{Unique, UniqueView, UniqueViewMut, World};
use wgpu::{
    BindGroup, BindGroupLayout, BlendComponent, ColorTargetState, ColorWrites,
    ComputePipeline, FragmentState, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, TextureFormat, TextureUsages, VertexState,
};

use crate::{
    graphics::{gpu::AbstractGpu, scene::Scene},
    scene::{asset_server::AssetServer, scene_state::SceneState},
    wgpu_graphics::{buffer::WGPUTexture, gpu::Gpu},
};

// TODO(Angel): Move this out wgpu graphics.
#[derive(Unique)]
pub struct SkyUpdater {
    texture_id: String,
    scene_id: String,
}

impl SkyUpdater {
    pub fn new(texture_id: String, scene_id: String) -> SkyUpdater {
        SkyUpdater {
            texture_id,
            scene_id,
        }
    }
}

#[derive(Unique)]
pub(crate) struct SkyPipeline {
    pub(crate) pipeline: RenderPipeline,

    pub(crate) texture_format: TextureFormat,
    pub(crate) equirect_layout: BindGroupLayout,
    pub(crate) equirectangular_conversion_pipeline: ComputePipeline,

    pub(crate) environment_layout: BindGroupLayout,
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

        let module = gpu.compile_program(
            "Equirectangular sky converter",
            include_str!("../shaders/equirectangular.wgsl"),
        );
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
        }
    }
}

/// Takes from the `AssetStorage` the indicated texture, generates the cubemap
/// based on it and stores it in the correct scene. Equirectangular texture -> cubemap.
pub(crate) fn sync_sky_pipeline_uniforms(world: &World) {
    let gpu = world.borrow::<UniqueView<AbstractGpu>>().unwrap();
    let asset_server = world.borrow::<UniqueView<AssetServer>>().unwrap();
    let sky_pipeline = world.borrow::<UniqueView<SkyPipeline>>().unwrap();

    // If there is a `SkyUpdater` component it means that the sky of some
    // scene must be updated.
    let sky_updater = match world.borrow::<UniqueView<SkyUpdater>>() {
        Ok(s_u) => s_u,
        _ => return,
    };

    // Extract the scene where the sky is updated.
    let scenes_states = world.borrow::<UniqueView<SceneState>>().unwrap();
    let scene = if let Some(scene) =
        scenes_states.sub_scenes.get(&sky_updater.scene_id)
    {
        scene
    } else {
        &scenes_states.main
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

    let scene_sky_texture = scene
        .sky_texture
        .downcast_ref::<WGPUTexture>()
        .expect("Incorrect texture type, expecting WGPUTexture");

    let dst_view =
        scene_sky_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("Sky cubemap copy view"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                // array_layer_count: Some(6),
                ..Default::default()
            });

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

    // TODO(Angel): 1080 is hardcoded we need to get that info from the scene descriptor.
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
