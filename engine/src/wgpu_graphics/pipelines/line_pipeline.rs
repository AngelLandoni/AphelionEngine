use shipyard::Unique;

use wgpu::{
    vertex_attr_array, BindGroupLayout, BlendComponent, BufferAddress,
    ColorTargetState, ColorWrites, DepthBiasState, DepthStencilState,
    FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState,
    RenderPipeline, RenderPipelineDescriptor, StencilState, VertexBufferLayout,
    VertexState,
};

use crate::{
    graphics::vertex::Vertex,
    wgpu_graphics::gpu::{Gpu, DEPTH_TEXTURE_FORMAT},
};

#[derive(Unique)]
pub struct DynamicMeshPipeline {
    /// Contains a reference to the pipeline.
    pub(crate) pipeline: RenderPipeline,
}

impl DynamicMeshPipeline {
    /// Creates and returns a new `DynamicMeshPipeline`.
    pub(crate) fn new(
        gpu: &Gpu,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> DynamicMeshPipeline {
        let program = gpu.compile_program(
            "dynamic_mesh_program",
            include_str!("../shaders/dynamic_mesh.wgsl"),
        );

        let layout =
            gpu.device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Dynamic mesh pipeline layout"),
                    bind_group_layouts: &[camera_bind_group_layout],
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

        DynamicMeshPipeline { pipeline }
    }
}
