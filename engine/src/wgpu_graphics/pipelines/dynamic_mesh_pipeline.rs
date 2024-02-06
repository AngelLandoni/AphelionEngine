use std::collections::HashMap;

use ahash::AHashMap;
use shipyard::{IntoIter, Unique, UniqueView, UniqueViewMut, View, World};

use wgpu::{
    vertex_attr_array, BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BlendComponent, Buffer, BufferAddress, BufferUsages, ColorTargetState, ColorWrites, DepthBiasState, DepthStencilState, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StencilState, VertexBufferLayout, VertexState
};

use crate::{
    app::App, graphics::{components::MeshComponent, gpu::AbstractGpu, vertex::Vertex}, scene::{asset_server::MeshResourceID, components::Transform}, schedule::Schedule, wgpu_graphics::{gpu::{Gpu, DEPTH_TEXTURE_FORMAT}, uniforms::CameraUniform}
};

#[derive(Unique)]
pub struct DynamicMeshPipeline {
    /// Contains a reference to the pipeline.
    pub(crate) pipeline: RenderPipeline,
    /// Conaints the associated bind group.
    pub(crate) camera_bind_group: BindGroup,
    /// Contains the buffer which holds the transform information.
    // TODO(Angel): Set this as u32, WGPU only supports u32 for instancing
    pub(crate) mesh_transform_buffers: AHashMap<MeshResourceID, (Buffer, u64)>,
}

impl DynamicMeshPipeline {
    /// Creates and returns a new `DynamicMeshPipeline`.
    pub(crate) fn new(app: &mut App, gpu: &Gpu) -> DynamicMeshPipeline {
        let program = gpu.compile_program(
            "triangle_test",
            include_str!("../shaders/triangle_test.wgsl"),
        );

        setup_schedulers(app);

        let camera_uniform = app.world
            .borrow::<UniqueView<CameraUniform>>()
            .expect("Unable to acquire camera uniform");

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

        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform.0.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let layout = gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Triangle test pipeline layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
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
            camera_bind_group,
            mesh_transform_buffers: AHashMap::new(),
        }
    }
}

/// Setups all the schedulers required by the pipeline.
fn setup_schedulers(app: &mut App) {
    app.schedule(Schedule::Update, |world| {
        world.run(sync_dynamic_entities_position)
    });
}

/// Recreates the transformation buffer and pass the transform information.
fn sync_dynamic_entities_position(
    gpu: UniqueView<AbstractGpu>,
    mut pipeline: UniqueViewMut<DynamicMeshPipeline>,
    transforms: View<Transform>,
    meshes: View<MeshComponent>,
) {
    let gpu = gpu
        .downcast_ref::<Gpu>()
        .expect("Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu");

    pipeline
        .mesh_transform_buffers
        .iter_mut()
        .for_each(|e| e.1.1 = 0);

    // TODO(Angel): Since we already know the maximum size per mesh, we can 
    // pre-allocate memory for each mesh to avoid dynamic reallocation during 
    // runtime, which can improve performance by reducing memory fragmentation 
    // and allocation overhead.
    let mut raw_transforms: AHashMap<MeshResourceID, Vec<u8>> = AHashMap::new();

    for ent in meshes.iter() {       
        pipeline.mesh_transform_buffers.entry(**ent).or_insert_with(|| {
            // Allocate the buffer.
            let buffer = gpu.allocate_aligned_zero_buffer(
                &format!("Mesh({}) transform", ent.0.0),
                // TODO(Angel): The size must be configured using the pipeline props.
                200000 * std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
            );
            (buffer, 0)
        });        
    }

    for (e, t) in (&meshes, &transforms).iter() {
        raw_transforms
            .entry(**e)
            .and_modify(|e| {
                let data = t.as_matrix_array();
                let a: &[u8] = bytemuck::cast_slice(&data);
                e.extend_from_slice(a);
            })
            .or_insert_with(|| { 
                let mut vec = Vec::new();
                let data = t.as_matrix_array();
                let a: &[u8] = bytemuck::cast_slice(&data);
                vec.extend_from_slice(a);
                vec
            });
    }

    for (m, b) in raw_transforms.iter() {       
        pipeline
            .mesh_transform_buffers
            .entry(*m)
            .and_modify(|e| {
                gpu
                    .queue
                    .write_buffer(&e.0, 0, b);
                e.1 = b.len() as u64 / Transform::raw_size();
            }
        );
    }
}
