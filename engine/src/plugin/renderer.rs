use egui::FontDefinitions;
use egui_wgpu_backend::RenderPass;
use egui_winit_platform::{Platform, PlatformDescriptor};

use shipyard::{Unique, UniqueView};
use winit::platform;

use crate::{
    app::App,
    graphics::{
        gpu::Gpu,
        CommandQueue,
        MAX_NUMBER_IF_COMMANDS_PER_FRAME
    },
    plugin::window::UniqueWindow,
    plugin::Pluggable
};


/// Shipyard component responsible for storing all renderer-related resources.
#[derive(Unique)]
pub struct UniqueRenderer(Gpu);

#[derive(Unique)]
pub struct UniqueCommandQueue(CommandQueue);

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

        let mut platform = Platform::new(PlatformDescriptor {
            physical_width: window_resource.host_window.descriptor.width,
            physical_height: window_resource.host_window.descriptor.width,
            scale_factor: 1.0,
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        
        let surface_format = gpu.surface.get_capabilities(&gpu.adapter).formats[0];
        RenderPass::new(&gpu.device, surface_format, 1);
        // Display the demo application that ships with egui.
        //let mut demo_app = egui_demo_lib::DemoWindows::default();

        drop(window_resource);

        app.world.add_unique(UniqueRenderer(gpu));

        app.world.add_unique(UniqueCommandQueue(
            CommandQueue::new(MAX_NUMBER_IF_COMMANDS_PER_FRAME)
        ));
    }
}