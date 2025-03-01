use shipyard::UniqueView;
use wgpu::{
    CommandEncoderDescriptor, Operations, RenderPassDepthStencilAttachment,
};

use crate::{
    graphics::gpu::AbstractGpu,
    scene::{assets::asset_server::AssetServer, scene_state::SceneState},
    wgpu_graphics::{
        buffer::{
            WGPUBindGroup, WGPUTexture, WgpuIndexBuffer, WgpuVertexBuffer,
        },
        gpu::Gpu,
        pipelines::dynamic_mesh_pipeline::DynamicMeshPipeline,
        CommandQueue, CommandSubmitOrder, OrderCommandBuffer,
    },
};

/// Renders the triangle test.
// TODO(Angel): Add support for sub scenes.
pub(crate) fn dynamic_mesh_pass_system(
    gpu: UniqueView<AbstractGpu>,
    dyn_mesh_pipeline: UniqueView<DynamicMeshPipeline>,
    queue: UniqueView<CommandQueue>,
    asset_server: UniqueView<AssetServer>,
    scenes: UniqueView<SceneState>,
) {
    let gpu = gpu.downcast_ref::<Gpu>().expect(
        "Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu",
    );

    let mut encoder =
        gpu.device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Dynamic mesh encoder"),
            });

    let main_meshes = scenes
        .main
        .mesh_transform_buffers
        .iter()
        .map(|(mesh_id, (buffer, count))| {
            (asset_server.load_mesh(mesh_id), buffer, count)
        })
        .collect::<Vec<_>>();

    let main_texture = scenes
        .main
        .target_texture
        .downcast_ref::<WGPUTexture>()
        .expect("The provided scene texture is not a WGPU texture");
    let depth_texture = scenes
        .main
        .depth_texture
        .downcast_ref::<WGPUTexture>()
        .expect("The provided scene depth texture is not a WGPU texture");

    let camera_bind_group = match &scenes.main.camera_bind_group {
        Some(bg) => bg,
        None => return,
    };

    let camera_bind_group = camera_bind_group
        .downcast_ref::<WGPUBindGroup>()
        .expect("Incorrect bind group type");

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Dynamic mesh pass"),
            color_attachments: &[
                // @location(0)
                Some(wgpu::RenderPassColorAttachment {
                    view: &main_texture.view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth_texture.view,
                depth_ops: Some(Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&dyn_mesh_pipeline.pipeline);
        pass.set_bind_group(0, &camera_bind_group.0, &[]);

        // Iterate over each mesh.
        for (mesh, t_buffer, count) in main_meshes
            .iter()
            // Only execute the draw if there are entities for the mesh.
            .filter(|(_, _, c)| **c >= 1)
        {
            let v_buffer = mesh
                .vertex_buffer
                .downcast_ref::<WgpuVertexBuffer>()
                .expect("Incorrect vertex buffer type, expecting WGPU vertex buffer");

            let i_buffer =
                mesh.index_buffer.downcast_ref::<WgpuIndexBuffer>().expect(
                    "Incorrect vertex buffer type, expecting WGPU index buffer",
                );

            let t_buffer = t_buffer.downcast_ref::<WgpuVertexBuffer>().expect(
                "Incorrect vertex buffer type, expecting WGPU vertex buffer",
            );

            pass.set_vertex_buffer(0, v_buffer.0.slice(..));
            pass.set_vertex_buffer(1, t_buffer.0.slice(..));
            pass.set_index_buffer(
                i_buffer.0.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            pass.draw_indexed(0..mesh.index_count, 0, 0..**count as u32);
        }
    }

    for (_id, scene) in &scenes.sub_scenes {
        let main_meshes = scene
            .mesh_transform_buffers
            .iter()
            .map(|(mesh_id, (buffer, count))| {
                (asset_server.load_mesh(mesh_id), buffer, count)
            })
            .collect::<Vec<_>>();

        let main_texture = scene
            .target_texture
            .downcast_ref::<WGPUTexture>()
            .expect("The provided scene texture is not a WGPU texture");
        let depth_texture = scene
            .depth_texture
            .downcast_ref::<WGPUTexture>()
            .expect("The provided scene depth texture is not a WGPU texture");

        let camera_bind_group = match &scene.camera_bind_group {
            Some(bg) => bg,
            None => return,
        };

        let camera_bind_group = camera_bind_group
            .downcast_ref::<WGPUBindGroup>()
            .expect("Incorrect bind group type");

        {
            let mut pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Sub pass dynamic mesh pass"),
                    color_attachments: &[
                        // @location(0)
                        Some(wgpu::RenderPassColorAttachment {
                            view: &main_texture.view,
                            resolve_target: None,
                            ops: Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                    ],
                    depth_stencil_attachment: Some(
                        RenderPassDepthStencilAttachment {
                            view: &depth_texture.view,
                            depth_ops: Some(Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        },
                    ),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            pass.set_pipeline(&dyn_mesh_pipeline.pipeline);
            pass.set_bind_group(0, &camera_bind_group.0, &[]);

            for (mesh, t_buffer, count) in main_meshes
                .iter()
                // Only execute the draw if there are entities for the mesh.
                .filter(|(_, _, c)| **c >= 1)
            {
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
                pass.set_index_buffer(
                    i_buffer.0.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                pass.draw_indexed(0..mesh.index_count, 0, 0..**count as u32);
            }
        }
    }

    let _ = queue.0.push(OrderCommandBuffer::new(
        Some("Render dynamic meshes".to_owned()),
        CommandSubmitOrder::DynamicMeshes,
        encoder.finish(),
    ));
}
