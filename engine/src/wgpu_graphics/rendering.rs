use shipyard::{UniqueView, UniqueViewMut};
use wgpu::CommandBuffer;

use crate::{
    graphics::{components::DepthTexture, gpu::AbstractGpu, BufferCreator},
    host::window::Window,
    scene::scene_state::SceneState,
    wgpu_graphics::{
        components::{ScreenFrame, ScreenTexture},
        gpu::Gpu,
        CommandQueue, OrderCommandBuffer,
    },
};

use super::{
    buffer::WGPUTexture,
    pipelines::frame_composition_pipeline::{
        create_frame_composition_texture_bind_group, FrameCompositionPipeline,
    },
};

/// DepthTexture: After the window is resized or the resolution changes the
/// depth texture must be updated to match resolutions.
pub(crate) fn reconfigure_main_textures_if_needed_system(
    mut gpu: UniqueViewMut<AbstractGpu>,
    window: UniqueView<Window>,
    mut depth_t: UniqueViewMut<DepthTexture>,
    mut s_state: UniqueViewMut<SceneState>,
    mut frame_conf: UniqueViewMut<FrameCompositionPipeline>,
) {
    let gpu = gpu.downcast_mut::<Gpu>().expect(
        "Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu",
    );

    if gpu.surface_config.width != window.size.width
        || gpu.surface_config.height != window.size.height
    {
        gpu.surface_config.width = window.size.width;
        gpu.surface_config.height = window.size.height;

        gpu.surface.configure(&gpu.device, &gpu.surface_config);

        depth_t.0 = gpu.allocate_depth_texture("Global depth texture", window.size.width, window.size.height);

        // Sync all the scenes which does not have a default resolution.
        // TODO(Angel): Add support for sub scenes.
        if s_state.main.should_sync_resolution_to_window {
            s_state.main.target_texture = gpu.allocate_target_texture(
                &s_state.main.label,
                window.size.width,
                window.size.height,
            );

            s_state.main.depth_texture = gpu.allocate_depth_texture(
                &s_state.main.label,
                window.size.width,
                window.size.height,
            );

            let main_texture = s_state
                .main
                .target_texture
                .downcast_ref::<WGPUTexture>()
                .expect("Incorrect Texture");

            let main_sampler = main_texture
                .sampler
                .as_ref()
                .expect("Unable to find sampler");

            let texture_bind_group =
                create_frame_composition_texture_bind_group(
                    gpu,
                    &frame_conf.texture_bind_group_layout,
                    &main_texture.view,
                    main_sampler,
                );

            frame_conf.texture_bind_group = Some(texture_bind_group);
        }
    }
}

/// Setups the screen texture into the world.
// TODO(Angel): Remove panic, to support headless.
pub(crate) fn acquire_screen_texture(
    gpu: UniqueView<AbstractGpu>,
    mut s_frame: UniqueViewMut<ScreenFrame>,
    mut s_texture: UniqueViewMut<ScreenTexture>,
) {
    let gpu = gpu.downcast_ref::<Gpu>().expect(
        "Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu",
    );

    if let Ok(frame) = gpu.surface.get_current_texture() {
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        s_frame.0 = Some(frame);
        s_texture.0 = Some(view);
    } else {
        panic!("Unable to acquire a new screen texture");
    }
}

// TODO(Angel): Remove panic, to support headless.
pub fn present_screen_texture(mut s_frame: UniqueViewMut<ScreenFrame>) {
    // `present` takes ownership of the frame so we need to take
    // it out. I assume it is becase the frame is ending and it
    // is not required any more and keeping it could lead to problems.
    let frame = std::mem::take(&mut s_frame.0);

    if let Some(frame) = frame {
        frame.present();
    } else {
        panic!("Unable to acquire texture frame");
    }
}

pub(crate) fn submit_commands_in_order(
    gpu: UniqueView<AbstractGpu>,
    c_queue: UniqueView<CommandQueue>,
) {
    let gpu = gpu.downcast_ref::<Gpu>().expect(
        "Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu",
    );

    let mut commands =
        Vec::<OrderCommandBuffer>::with_capacity(c_queue.0.len());

    while let Some(c) = c_queue.0.pop() {
        commands.push(c);
    }

    commands.sort_by_key(|c| c.order);

    let mut wgpu_commands = Vec::<CommandBuffer>::new();
    while let Some(c) = commands.pop() {
        wgpu_commands.push(c.command);
    }

    gpu.queue.submit(wgpu_commands);
}
