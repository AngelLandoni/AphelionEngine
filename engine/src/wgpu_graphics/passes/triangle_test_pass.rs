use std::sync::Arc;

use shipyard::UniqueView;
use wgpu::{Buffer, CommandEncoderDescriptor, Operations};

use crate::{
    graphics::{
        mesh::Mesh,
        gpu::AbstractGpu
    },
    scene::asset_server::AssetServer,
    wgpu_graphics::{
        buffer::{WgpuIndexBuffer, WgpuVertexBuffer},
        components::ScreenTexture,
        gpu::Gpu,
        pipelines::traingle_test_pipeline::TriangleTestPipeline,
        CommandQueue,
        CommandSubmitOrder,
        OrderCommandBuffer
    }
};

/// Renders the triangle test.
pub(crate) fn triangle_test_pass_system(
    gpu: UniqueView<AbstractGpu>,
    triangle_pipeline: UniqueView<TriangleTestPipeline>,
    screen_texture: UniqueView<ScreenTexture>,
    queue: UniqueView<CommandQueue>,
    asset_server: UniqueView<AssetServer>,
) {
    let gpu = gpu
        .downcast_ref::<Gpu>()
        .expect("Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu");

    let s_texture = match &screen_texture.0 {
        Some(s_t) => s_t,
        None => return,
    };

    let mut encoder = gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Tirangle test encoder")
        });
    
    let preload_meshes: Vec<(Arc<Mesh>, &Buffer, &u32)> = triangle_pipeline
        .mesh_transform_buffers
        .iter()
        .map(|(mesh_id, (buffer, count))| {
            (asset_server.load_mesh(mesh_id), buffer, count)
        })
        .collect::<Vec<_>>();

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Tirangle test pass"),
            color_attachments: &[

                // @location(0)
                Some(wgpu::RenderPassColorAttachment {
                    view: s_texture,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })

            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None 
        });

        pass.set_pipeline(&triangle_pipeline.pipeline);
        // Camera position.
        pass.set_bind_group(0, &triangle_pipeline.camera_bind_group, &[]);

        for (mesh, buffer, count) in preload_meshes.iter() {
            let v_buffer = mesh
                .vertex_buffer
                .downcast_ref::<WgpuVertexBuffer>()
                .expect("Incorrect vertex buffer type, expecting WGPU vertex buffer");

            let i_buffer = mesh
                .index_buffer
                .downcast_ref::<WgpuIndexBuffer>()
                .expect("Incorrect vertex buffer type, expecting WGPU index buffer");

            pass.set_vertex_buffer(0, v_buffer.0.slice(..));
            pass.set_vertex_buffer(1, buffer.slice(..));
            pass.set_index_buffer(i_buffer.0.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..mesh.index_count, 0, 0..**count);
        }

        /*for (index, mesh) in ents.iter().enumerate() {
            pass.set_bind_group(1, &triangle_pipeline.transform_bind_group, &[]);

            let v_buffer = mesh
                .vertex_buffer
                .downcast_ref::<WgpuVertexBuffer>()
                .expect("Incorrect vertex buffer type, expecting WGPU vertex buffer");

            let i_buffer = mesh
                .index_buffer
                .downcast_ref::<WgpuIndexBuffer>()
                .expect("Incorrect vertex buffer type, expecting WGPU index buffer");

            pass.set_vertex_buffer(0, v_buffer.0.slice(..));
            pass.set_index_buffer(i_buffer.0.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }*/
    }
    
    let _ = queue.0.push(OrderCommandBuffer::new(
        Some("Render egui".to_owned()),
        CommandSubmitOrder::TriangleTest,
        encoder.finish(),
    ));
}