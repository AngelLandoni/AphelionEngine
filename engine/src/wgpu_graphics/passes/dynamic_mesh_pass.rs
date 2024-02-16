use shipyard::{Unique, UniqueView};
use wgpu::{CommandEncoderDescriptor, Operations, RenderPassDepthStencilAttachment};

use crate::{
    graphics::{components::DepthTexture, gpu::AbstractGpu},
    scene::{asset_server::AssetServer, scene_state::SceneState},
    wgpu_graphics::{
        buffer::{WGPUTexture, WgpuIndexBuffer, WgpuVertexBuffer}, components::ScreenTexture, gpu::Gpu, pipelines::dynamic_mesh_pipeline::DynamicMeshPipeline, CommandQueue, CommandSubmitOrder, OrderCommandBuffer
    },
};

/// Renders the triangle test.
// TODO(Angel): Add support for sub scenes.
pub(crate) fn dynamic_mesh_pass_system(
    gpu: UniqueView<AbstractGpu>,
    depth_texture: UniqueView<DepthTexture>,
    dyn_mesh_pipeline: UniqueView<DynamicMeshPipeline>,
    queue: UniqueView<CommandQueue>,
    asset_server: UniqueView<AssetServer>,
    scenes: UniqueView<SceneState>,
) {
    let gpu = gpu.downcast_ref::<Gpu>().expect(
        "Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu",
    );

    let mut encoder = gpu.device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Dynamic mesh encoder"),
        });

    // Main scene.

    let main_meshes = scenes
        .main
        .mesh_transform_buffers
        .iter()
        .map(|(mesh_id, (buffer, count))| (asset_server.load_mesh(mesh_id), buffer, count))
        .collect::<Vec<_>>();
    
    let main_texture = scenes.main.target_texture.downcast_ref::<WGPUTexture>().expect("The provided scene texture is not a WGPU texture");
    let depth_texture = scenes.main.depth_texture.downcast_ref::<WGPUTexture>().expect("The provided scene depth texture is not a WGPU texture");

    let main_camera_bind_group = match &dyn_mesh_pipeline.camera_bind_group {
        Some(bg) => bg,
        None => return,
    };

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Dynamic mesh pass"),
            color_attachments: &[
                // @location(0)
                Some(wgpu::RenderPassColorAttachment {
                    view: &main_texture.view,
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
                view: &depth_texture.view,
                depth_ops: Some(Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&dyn_mesh_pipeline.pipeline);
        pass.set_bind_group(0, main_camera_bind_group, &[]);

        for (mesh, t_buffer, count) in main_meshes.iter() {
            let v_buffer = mesh
                .vertex_buffer
                .downcast_ref::<WgpuVertexBuffer>()
                .expect("Incorrect vertex buffer type, expecting WGPU vertex buffer");

            let i_buffer = mesh
                .index_buffer
                .downcast_ref::<WgpuIndexBuffer>()
                .expect("Incorrect vertex buffer type, expecting WGPU index buffer");

            let t_buffer = t_buffer
                .downcast_ref::<WgpuVertexBuffer>()
                .expect("Incorrect vertex buffer type, expecting WGPU vertex buffer");

            pass.set_vertex_buffer(0, v_buffer.0.slice(..));
            pass.set_vertex_buffer(1, t_buffer.0.slice(..));
            pass.set_index_buffer(i_buffer.0.slice(..), wgpu::IndexFormat::Uint16);
            //pass.draw_indexed(0..mesh.index_count, 0, 0..**count as u32 + 1);
            pass.draw_indexed(0..mesh.index_count, 0, 0..**count as u32);
        }
    }

    let _ = queue.0.push(OrderCommandBuffer::new(
        Some("Render dynamic meshes".to_owned()),
        CommandSubmitOrder::DynamicMeshes,
        encoder.finish(),
    ));

    /*
    let preload_meshes: Vec<(Arc<Mesh>, &Buffer, &u64)> = triangle_pipeline
        .mesh_transform_buffers
        .iter()
        .map(|(mesh_id, (buffer, count))| (asset_server.load_mesh(mesh_id), buffer, count))
        .collect::<Vec<_>>();

    // for scene in scenes {

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

    // }

    let _ = queue.0.push(OrderCommandBuffer::new(
        Some("Render egui".to_owned()),
        CommandSubmitOrder::TriangleTest,
        encoder.finish(),
    ));*/
}
