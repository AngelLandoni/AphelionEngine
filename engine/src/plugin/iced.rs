use iced::{Font, Pixels};
use iced_wgpu::{
    graphics::Viewport,
    Renderer,
    Backend,
    Settings
};
use iced_winit::runtime::{Debug, program};
use shipyard::UniqueView;

use crate::{
    app::App,
    plugin::Pluggable, graphics::components::UniqueRenderer,
};

pub struct IcedPlugin;

impl Pluggable for IcedPlugin {
    fn configure(&self, app: &mut App) {
        let uniqueGpu = app
            .world
            .borrow::<UniqueView<UniqueRenderer>>()
            .expect("Unable to adquire GPU");

        let physical_size = window.inner_size();
        let mut viewport = Viewport::with_physical_size(v
            Size::new(physical_size.width, physical_size.height),v
            window.scale_factor(),v
        );

        let mut debug = Debug::new();
        let renderer = Renderer::new(
            Backend::new(
                &uniqueGpu.gpu.device,
                &uniqueGpu.gpu.queue,
                Settings::default(),
                uniqueGpu.gpu.texture_format,
            ),
            Font::default(),
            Pixels(16.0),
        );

        let mut state = program::State::new(
            controls,
            viewport.logical_size(),
            &mut renderer,
            &mut debug,
        );
    }
}