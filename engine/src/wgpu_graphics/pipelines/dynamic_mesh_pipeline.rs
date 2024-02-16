use ahash::AHashMap;
use shipyard::{Unique, UniqueView, UniqueViewMut};

use wgpu::{
    vertex_attr_array, BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BlendComponent, BufferAddress, ColorTargetState, ColorWrites, DepthBiasState, DepthStencilState, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StencilState, VertexBufferLayout, VertexState
};

use crate::{
    graphics::{gpu::AbstractGpu, vertex::Vertex}, scene::scene_state::SceneState, wgpu_graphics::{buffer::WgpuUniformBuffer, gpu::{Gpu, DEPTH_TEXTURE_FORMAT}}
};

#[derive(Unique)]
pub struct DynamicMeshPipeline {
    /// Contains a reference to the pipeline.
    pub(crate) pipeline: RenderPipeline,

    // TODO(Angel): Probably move this to the `Scene` type?.
    pub(crate) camera_bind_group_layout: BindGroupLayout,
    pub(crate) camera_bind_group: Option<BindGroup>,

    pub(crate) scenes_camera_bind_group: AHashMap<String, BindGroup>,
}

impl DynamicMeshPipeline {
    /// Creates and returns a new `DynamicMeshPipeline`.
    pub(crate) fn new(gpu: &Gpu) -> DynamicMeshPipeline {
        let program = gpu.compile_program(
            "dynamic_mesh_program",
            include_str!("../shaders/triangle_test.wgsl"),
        );

        let camera_bind_group_layout =
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
                });

        let layout =
            gpu.device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Dynamic mesh pipeline layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = gpu
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Dynamic mesh render pipeline"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &program,
                    entry_point: "vs_main",
                    buffers: &[
                        // Defines the `Vertex` layout format.
                        VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                        },
                        // Defines the Vertex transform.
                        VertexBufferLayout {
                            array_stride: std::mem::size_of::<[[f32; 4]; 4]>() as BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &vertex_attr_array![
                                2 => Float32x4,
                                3 => Float32x4,
                                4 => Float32x4,
                                5 => Float32x4,
                            ],
                        },
                    ],
                },
                primitive: PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: DEPTH_TEXTURE_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
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

        DynamicMeshPipeline {
            pipeline,
            camera_bind_group_layout,
            camera_bind_group: None,
            scenes_camera_bind_group: AHashMap::new(),
         }
    }
}

pub(crate) fn setup_dynamic_mesh_pipelines_uniforms_system(
    gpu: UniqueView<AbstractGpu>,
    mut dyn_mesh_pipeline: UniqueViewMut<DynamicMeshPipeline>,
    s_state: UniqueView<SceneState>,
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

    dyn_mesh_pipeline.camera_bind_group = Some(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &dyn_mesh_pipeline.camera_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.0.as_entire_binding(),
            },
        ],
        label: Some("frame_composition_diffuse_bind_group"),
    }));

    for (id, scene) in &s_state.sub_scenes {

        let camera_buffer = scene
            .camera_buffer
            .downcast_ref::<WgpuUniformBuffer>()
            .expect("Incorrect uniform buffer type");

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &dyn_mesh_pipeline.camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.0.as_entire_binding(),
                },
            ],
            label: Some("frame_composition_diffuse_bind_group"),
        });

        dyn_mesh_pipeline.scenes_camera_bind_group.insert(
            id.to_owned(),
            bind_group,
        );
    }
}

