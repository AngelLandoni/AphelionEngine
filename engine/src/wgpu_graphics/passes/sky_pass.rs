use std::iter;

use log::warn;
use shipyard::UniqueView;
use wgpu::{CommandEncoderDescriptor, Operations};

use crate::{
    graphics::gpu::AbstractGpu,
    scene::scene_state::SceneState,
    wgpu_graphics::{
        buffer::{WGPUBindGroup, WGPUTexture},
        components::{ScreenFrame, ScreenTexture},
        gpu::Gpu,
        pipelines::{
            frame_composition_pipeline::FrameCompositionPipeline,
            sky_pipeline::SkyPipeline,
        },
        CommandQueue, CommandSubmitOrder, OrderCommandBuffer,
    },
};

pub(crate) fn sky_pass_system(
    gpu: UniqueView<AbstractGpu>,
    s_state: UniqueView<SceneState>,
    sky_pipeline: UniqueView<SkyPipeline>,
    queue: UniqueView<CommandQueue>,
) {
    let gpu = gpu.downcast_ref::<Gpu>().expect(
        "Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu",
    );

    let mut encoder =
        gpu.device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Frame composition encoder"),
            });

    for (_, scene) in s_state
        .sub_scenes
        .iter()
        .chain(iter::once((&"".to_owned(), &s_state.main)))
    {
        let target_texture = scene
            .target_texture
            .downcast_ref::<WGPUTexture>()
            .expect("The provided scene texture is not a WGPU Texture");

        let camera_bind_group = match &scene.camera_bind_group {
            Some(bg) => bg,
            None => return,
        };

        let camera_bind_group = camera_bind_group
            .downcast_ref::<WGPUBindGroup>()
            .expect("Incorrect bind group type");

        let sky_texture_bind_group = match &scene.sky_env_bind_group {
            Some(bg) => bg,
            None => {
                warn!("Missing sky texture_bind_group!");
                return;
            }
        };

        let sky_texture_bind_group = sky_texture_bind_group
            .downcast_ref::<WGPUBindGroup>()
            .expect("Incorrect bind group type");

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Frame composition pass"),
            color_attachments: &[
                // @location(0)
                Some(wgpu::RenderPassColorAttachment {
                    view: &target_texture.view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&sky_pipeline.pipeline);
        pass.set_bind_group(0, &camera_bind_group.0, &[]);
        pass.set_bind_group(1, &sky_texture_bind_group.0, &[]);
        pass.draw(0..3, 0..1);
    }

    let _ = queue.0.push(OrderCommandBuffer::new(
        Some("Copy final texture to screen".to_owned()),
        CommandSubmitOrder::FrameComposition,
        encoder.finish(),
    ));
}
