use wgpu::{
    vertex_attr_array, BindGroupLayout, BlendComponent, BufferAddress,
    ColorTargetState, ColorWrites, DepthBiasState, DepthStencilState,
    FragmentState, MultisampleState, PrimitiveState, RenderPipeline,
    ShaderModule, StencilState, VertexBufferLayout,
};

use crate::{
    graphics::vertex::Vertex,
    wgpu_graphics::gpu::{Gpu, DEPTH_TEXTURE_FORMAT},
};

/// Creates a new forward pipeline using the provided information.
pub(crate) fn create_forward_pipeline(
    gpu: &Gpu,
    camera_bind_group_layout: &BindGroupLayout,
    material_bindgroup_layouts: Option<&BindGroupLayout>,
    fragment_shader: &ShaderModule,
) -> RenderPipeline {
    // The vertex shader used by all forward passes remains constant,
    // as it handles common vertex transformations. The fragment shader,
    // on the other hand, allows for dynamic changes based on Material properties.
    let vertex_shader = gpu.compile_program(
        "Forward renderer vertex shader",
        include_str!("../shaders/forward_vertex.wgsl"),
    );

    // Combine the camera bindgroup and the material ones.
    let mut bind_group_layouts = vec![camera_bind_group_layout];
    if let Some(material_bindgroup_layouts) = material_bindgroup_layouts {
        bind_group_layouts.push(material_bindgroup_layouts);
    }

    // Creates the new pipeline using the provided bind groups layouts.
    let pipeline_layout =
        gpu.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Forward renderer pipeline layout"),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            });

    let pipeline = gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Forward renderer pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
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
                    }
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
                module: fragment_shader,
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

    pipeline
}
