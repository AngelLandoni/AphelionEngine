use std::sync::Arc;

use shipyard::UniqueView;
use wgpu::{Buffer, CommandEncoderDescriptor, Operations, RenderPassDepthStencilAttachment};

use crate::{
    graphics::{components::DepthTexture, gpu::AbstractGpu, mesh::Mesh},
    scene::asset_server::AssetServer,
    wgpu_graphics::{
        buffer::{WGPUTexture, WgpuIndexBuffer, WgpuVertexBuffer},
        components::ScreenTexture,
        gpu::Gpu,
        pipelines::dynamic_mesh_pipeline::DynamicMeshPipeline,
        CommandQueue, CommandSubmitOrder, OrderCommandBuffer,
    },
};

/// Renders the triangle test.
pub(crate) fn dynamic_mesh_pass_system(
    gpu: UniqueView<AbstractGpu>,
    depth_texture: UniqueView<DepthTexture>,
    triangle_pipeline: UniqueView<DynamicMeshPipeline>,
    screen_texture: UniqueView<ScreenTexture>,
    queue: UniqueView<CommandQueue>,
    asset_server: UniqueView<AssetServer>,
) {
    let gpu = gpu
        .downcast_ref::<Gpu>()
        .expect("Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu");

    let d_texture = depth_texture
        .downcast_ref::<WGPUTexture>()
        .expect("Incorrect depth Texture type, expecting WGPU depth texture");

    let s_texture = match &screen_texture.0 {
        Some(s_t) => s_t,
        None => return,
    };

    let mut encoder = gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Tirangle test encoder"),
        });

    let preload_meshes: Vec<(Arc<Mesh>, &Buffer, &u64)> = triangle_pipeline
        .mesh_transform_buffers
        .iter()
        .map(|(mesh_id, (buffer, count))| (asset_server.load_mesh(mesh_id), buffer, count))
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
                }),
            ],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &d_texture.view,
                depth_ops: Some(Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
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
            //pass.draw_indexed(0..mesh.index_count, 0, 0..**count as u32 + 1);
            pass.draw_indexed(0..mesh.index_count, 0, 0..**count as u32);
        }
    }

    let _ = queue.0.push(OrderCommandBuffer::new(
        Some("Render egui".to_owned()),
        CommandSubmitOrder::TriangleTest,
        encoder.finish(),
    ));
}