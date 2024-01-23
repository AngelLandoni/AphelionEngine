use shipyard::Unique;
use wgpu::{
    RenderPipeline,
    PipelineLayoutDescriptor,
    RenderPipelineDescriptor,
    VertexState,
    PrimitiveState,
    MultisampleState,
    FragmentState,
    ColorTargetState,
    BlendComponent,
    ColorWrites,
    BindGroupLayoutDescriptor,
    BindGroupLayoutEntry,
    ShaderStages,
    Buffer, BindGroup
};

use crate::graphics::gpu::Gpu;

#[derive(Unique)]
pub struct TriangleTestPipeline {
    /// Contains a reference to the pipeline.
    pub(crate) pipeline: RenderPipeline,
    /// Conaints the associated bind group.
    pub(crate) camera_bind_group: BindGroup,
}

impl TriangleTestPipeline {
    /// Creates and returns a new `TriangleTestPipeline`.
    pub(crate) fn new(gpu: &Gpu, camera_buffer: &Buffer) -> TriangleTestPipeline {
        let program = gpu.compile_program(
            "triangle_test",
            include_str!("../shaders/triangle_test.wgsl")
        );

        let camera_bind_group_layout = gpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera bind group"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                }
            ],
        });

        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let layout = gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Triangle test pipeline layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout
                ],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Triangle test render pipeline"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &program,
                    entry_point: "vs_main",
                    buffers: &[],
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
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(FragmentState {
                    module: &program,
                    entry_point: "fs_main",
                    targets: &[
                        Some(ColorTargetState {
                            format: gpu.surface_config.format,
                            blend: Some(wgpu::BlendState { 
                                color: BlendComponent::REPLACE,
                                alpha: BlendComponent::REPLACE,
                            }),
                            write_mask: ColorWrites::ALL,
                        })
                    ],
                }),
                multiview: None,
            });
        
        TriangleTestPipeline {
            pipeline,
            camera_bind_group,
        }
    }
}
