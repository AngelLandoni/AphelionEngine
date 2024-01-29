use shipyard::{UniqueView, UniqueViewMut};
use wgpu::CommandBuffer;

use crate::{
    graphics::{
        CommandQueue,
        OrderCommandBuffer,
        components::{
            UniqueRenderer,
            ScreenFrame,
            ScreenTexture,
        }
    },
    host::window::Window
};

pub fn reconfigure_surface_if_needed_system(
    mut gpu: UniqueViewMut<UniqueRenderer>,
    window: UniqueView<Window>
) {
    if gpu.gpu.surface_config.width != window.size.width ||
       gpu.gpu.surface_config.height != window.size.height {

        gpu.gpu.surface_config.width = window.size.width;
        gpu.gpu.surface_config.height = window.size.height;

        gpu.gpu.surface.configure(&gpu.gpu.device, &gpu.gpu.surface_config);
    }
}

/// Setups the screen texture into the world.
// TODO(Angel): Remove panic, to support headless.
pub fn acquire_screen_texture(u_gpu: UniqueView<UniqueRenderer>, 
                              mut s_frame: UniqueViewMut<ScreenFrame>,
                              mut s_texture: UniqueViewMut<ScreenTexture>) {
    if let Ok(frame) = u_gpu.gpu.surface.get_current_texture() {
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

pub fn submit_commands_in_order(u_gpu: UniqueView<UniqueRenderer>,
                                c_queue: UniqueView<CommandQueue>) {
    let mut commands = Vec::<OrderCommandBuffer>::with_capacity(
        c_queue.0.len(),
    );

    while let Some(c) = c_queue.0.pop() {
        commands.push(c);
    }

    commands.sort_by_key(|c| c.order);

    let mut wgpu_commands = Vec::<CommandBuffer>::new();
    while let Some(c) = commands.pop() {
        wgpu_commands.push(c.command);
    }

    u_gpu.gpu.queue.submit(wgpu_commands);
}