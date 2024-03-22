use shipyard::UniqueView;
use wgpu::{CommandEncoderDescriptor, Operations};

use crate::{
    graphics::{
        components::{ScreenFrame, ScreenTexture},
        gpu::{AbstractGpu, Gpu},
        pipeline::frame_composition_pipeline::FrameCompositionPipeline,
        CommandQueue, CommandSubmitOrder, OrderCommandBuffer,
    },
    scene::scene_state::SceneState,
};

pub(crate) fn frame_composition_pass_system(
    gpu: UniqueView<AbstractGpu>,
    _s_state: UniqueView<SceneState>,
    _screen_frame: UniqueView<ScreenFrame>,
    pipeline: UniqueView<FrameCompositionPipeline>,
    screen_texture: UniqueView<ScreenTexture>,
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

    let s_texture = match &screen_texture.0 {
        Some(s_t) => s_t,
        None => return,
    };

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Frame composition pass"),
            color_attachments: &[
                // @location(0)
                Some(wgpu::RenderPassColorAttachment {
                    view: s_texture,
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

        if let Some(bind_group) = &pipeline.texture_bind_group {
            pass.set_bind_group(0, bind_group, &[]);
        }
        pass.set_pipeline(&pipeline.pipeline);
        pass.draw(0..6, 0..1);
    }

    let _ = queue.0.push(OrderCommandBuffer::new(
        Some("Copy final texture to screen".to_owned()),
        CommandSubmitOrder::FrameComposition,
        encoder.finish(),
    ));
}
