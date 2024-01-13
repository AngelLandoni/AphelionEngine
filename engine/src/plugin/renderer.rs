use shipyard::UniqueView;

use crate::{
    app::App,
    graphics::{
        CommandQueue,
        gpu::Gpu,
        components::{
            UniqueRenderer,
            UniqueCommandQueue
        },
        MAX_NUMBER_IF_COMMANDS_PER_FRAME
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

        app.world.add_unique(UniqueRenderer {
            gpu
        });

        app.world.add_unique(UniqueCommandQueue {
            queue: CommandQueue::new(MAX_NUMBER_IF_COMMANDS_PER_FRAME)
        });
    }
}