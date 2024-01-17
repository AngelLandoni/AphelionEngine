use shipyard::{UniqueView, UniqueViewMut};

use crate::{
    app::App,
    schedule::Schedule,
    graphics::{
        gpu::Gpu,
        components::{
            UniqueRenderer,
            ScreenTexture,
            ScreenFrame,
        },
        rendering::{
            acquire_screen_texture,
            present_screen_texture, submit_commands_in_order
        },
        CommandQueue,
        OrderCommandQueue,
        MAX_NUMBER_IF_COMMANDS_PER_FRAME,
    },
    host::components::UniqueWindow,
    plugin::Pluggable,
};

pub struct WgpuRendererPlugin;

impl Pluggable for WgpuRendererPlugin {
    fn configure(&self, app: &mut App) {
        let window_resource = app
            .world
            .borrow::<UniqueView<UniqueWindow>>()
            .expect("Configure the window context before setting up the renderer"); 

        let gpu = futures_lite::future::block_on(
            Gpu::new(&window_resource.host_window)
        );

        drop(window_resource);

        // Setup all the unique resources.
        {
            let world = &app.world;
            world.add_unique(ScreenTexture(None));

            world.add_unique(UniqueRenderer {
                gpu
            });

            world.add_unique(ScreenFrame(None));
            world.add_unique(ScreenTexture(None));
    
            world.add_unique(CommandQueue(
                OrderCommandQueue::new(MAX_NUMBER_IF_COMMANDS_PER_FRAME)
            ));
        }
        
        // Setup scheludes.
        {
            app.schedule(Schedule::InitFrame, |world| {
                world.run(acquire_screen_texture);
            });

            app.schedule(Schedule::QueueSubmit, |world| {
                world.run(submit_commands_in_order);
            });

            app.schedule(Schedule::EndFrame, |world| {
                world.run(present_screen_texture);
                world.run(|mut texture: UniqueViewMut<ScreenTexture>| {
                    texture.0 = None;
                })
            });
        }
    }
}