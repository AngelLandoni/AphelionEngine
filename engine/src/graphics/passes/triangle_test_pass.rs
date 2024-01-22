use shipyard::UniqueView;
use wgpu::{CommandEncoderDescriptor, Operations};

use crate::graphics::{
    pipelines::traingle_test_pipeline::TriangleTestPipeline,
    components::{UniqueRenderer, ScreenTexture}, CommandQueue, CommandSubmitOrder, OrderCommandBuffer
};

/// Renders the triangle test.
pub fn triangle_test_pass_system(
    gpu: UniqueView<UniqueRenderer>,
    triangle_pipeline: UniqueView<TriangleTestPipeline>,
    screen_texture: UniqueView<ScreenTexture>,
    queue: UniqueView<CommandQueue>
) {
    let s_texture = match &screen_texture.0 {
        Some(s_t) => s_t,
        None => return,
    };

    let mut encoder = gpu
        .gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Tirangle test encoder")
        });

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Tirangle test pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: s_texture,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None 
        });

        pass.set_pipeline(&triangle_pipeline.pipeline);
        pass.draw(0..3, 0..1);
    }
    
    let _ = queue.0.push(OrderCommandBuffer::new(
        Some("Render egui".to_owned()),
        CommandSubmitOrder::TriangleTest,
        encoder.finish(),
    ));
}