use shipyard::Unique;

use wgpu::{
    BindGroupLayout, ColorTargetState, FragmentState, PipelineLayoutDescriptor,
    RenderPipeline, RenderPipelineDescriptor, VertexState,
};

use crate::wgpu_graphics::gpu::Gpu;

#[derive(Unique)]
pub struct InfiniteGridPipeline {
    /// Contains a reference to the pipeline.
    pub(crate) pipeline: RenderPipeline,
}

impl InfiniteGridPipeline {
    /// Creates and returns a new `InfiniteGridPipeline`.
    pub(crate) fn new(
        gpu: &Gpu,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let program = gpu.compile_program(
            "infinite_grid_composition",
            include_str!("../shaders/infinite_grid.wgsl"),
        );

        let layout =
            gpu.device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Infinite grid pipeline layout"),
                    bind_group_layouts: &[camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline =
            gpu.device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("Infinite grid render pipeline"),
                    layout: Some(&layout),
                    vertex: VertexState {
                        module: &program,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(FragmentState {
                        module: &program,
                        entry_point: "fs_main",
                        targets: &[Some(ColorTargetState {
                            format: gpu.surface_config.format,
                            blend: Some(
                                wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                            ),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                });

        InfiniteGridPipeline { pipeline }
    }
}
