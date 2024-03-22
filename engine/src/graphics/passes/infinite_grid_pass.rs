use shipyard::UniqueView;
use wgpu::{
    CommandEncoderDescriptor, Operations, RenderPassDepthStencilAttachment,
};

use crate::{
    graphics::{
        buffer::{WGPUBindGroup, WGPUTexture},
        gpu::{AbstractGpu, Gpu},
        pipeline::infinite_grid_pipeline::InfiniteGridPipeline,
        CommandQueue, CommandSubmitOrder, OrderCommandBuffer,
    },
    scene::scene_state::SceneState,
};

pub(crate) fn infinite_grid_pass_system(
    gpu: UniqueView<AbstractGpu>,
    pipeline: UniqueView<InfiniteGridPipeline>,
    scenes: UniqueView<SceneState>,
    queue: UniqueView<CommandQueue>,
) {
    let gpu = gpu.downcast_ref::<Gpu>().expect(
        "Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu",
    );

    let mut encoder =
        gpu.device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Infinite grid encoder"),
            });

    // Iterate over each scene and draw the grid only if it is requested.
    for (_id, scene) in &scenes.sub_scenes {
        let camera_bind_group = match &scene.camera_bind_group {
            Some(c) => c,
            None => continue,
        };

        let camera_bind_group = camera_bind_group
            .downcast_ref::<WGPUBindGroup>()
            .expect("Incorrect bind group type");

        let target_texture = scene
            .target_texture
            .downcast_ref::<WGPUTexture>()
            .expect("The provided scene texture is not a WGPU texture");

        let depth_texture = scene
            .depth_texture
            .downcast_ref::<WGPUTexture>()
            .expect("The provided scene texture is not a WGPU texture");

        {
            let mut pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(&format!(
                        "Infinite grid pass, {}",
                        scene.label
                    )),
                    color_attachments: &[
                        // @location(0)
                        Some(wgpu::RenderPassColorAttachment {
                            view: &target_texture.view,
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

            pass.set_pipeline(&pipeline.pipeline);
            pass.set_bind_group(0, &camera_bind_group.0, &[]);
            pass.draw(0..6, 0..1)
        }
    }

    let _ = queue.0.push(OrderCommandBuffer::new(
        Some("Copy final texture to screen".to_owned()),
        CommandSubmitOrder::DebugGrid,
        encoder.finish(),
    ));
}
