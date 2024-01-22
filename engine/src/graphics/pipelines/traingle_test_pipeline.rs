use shipyard::Unique;
use wgpu::{RenderPipeline, PipelineLayoutDescriptor, RenderPipelineDescriptor, VertexState, PrimitiveState, MultisampleState, FragmentState, ColorTargetState, BlendComponent, Color, ColorWrites};

use crate::graphics::gpu::Gpu;

#[derive(Unique)]
pub struct TriangleTestPipeline {
    /// Contains a reference to the pipeline.
    pub(crate) pipeline: RenderPipeline,
}

impl TriangleTestPipeline {
    /// Creates and returns a new `TriangleTestPipeline`.
    pub(crate) fn new(gpu: &Gpu) -> TriangleTestPipeline {
        let program = gpu.compile_program(
            "triangle_test",
            include_str!("../shaders/triangle_test.wgsl")
        );

        let layout = gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Triangle test pipeline layout"),
                bind_group_layouts: &[],
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
        }
    }
}
