use winit::{
    event_loop::EventLoop,
    window::WindowBuilder, 
    event::{Event, WindowEvent},
    dpi::LogicalSize,
};

use crate::{
    plugin::Pluggable,
    app::App,
};

pub struct WinitWindowPlugin;

impl Pluggable for WinitWindowPlugin {
    /// Spawns the main window and triggers the `winit` run loop.
    fn configure(&self, app: &mut App) {
        let event_loop = EventLoop::new()
            .expect("Unable to initialize `Winit` main run loop");

        app.set_run_loop(move |app: &mut App| {
            // Create a window builder
            let window_builder = WindowBuilder::new()
                .with_title("My Window")
                .with_inner_size(LogicalSize::new(800.0, 600.0));

            // Create the window
            let _window = window_builder.build(&event_loop)
                .expect("Unable to spawn main `Winit` `Window`");

            event_loop.run(move |event, elwt| {
                let (e, _w_id) = match event {
                    Event::WindowEvent { window_id, event } => {
                        (event, window_id)
                    }

                    _ => {
                        app.update();
                        return;
                    }
                };

                match e {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }

                    _ => (),
                };
            })
            .expect("Unable to lunch `Winit` event loop");
        });
    }
}