use std::sync::{Arc, Mutex};

use raw_window_handle::{
    HasRawWindowHandle,
    HasRawDisplayHandle
};

use winit::{
    event_loop::EventLoop,
    window::WindowBuilder, 
    event::{Event, WindowEvent},
    dpi::LogicalSize, 
};

use crate::{
    plugin::Pluggable,
    app::App, 
    host::{
        components::UniqueWindow,
        window::{
            Window,
            WindowInfoAccessible
        }
    },
    types::Size,
};

pub struct WinitWindowWrapper(winit::window::Window);

impl WindowInfoAccessible for WinitWindowWrapper {
    fn inner_size(&self) -> Size<u32> {
        Size::new(self.0.inner_size().width, self.0.inner_size().height)        
    }

    fn scale_factor(&self) -> f64 {
        self.0.scale_factor()
    }
}

pub struct WinitWindowPlugin {
    title: String,
    width: u32,
    height: u32,
}

impl WinitWindowPlugin {
    /// Creates a new `Winit` Window.
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        WinitWindowPlugin {
            title: title.to_string(),
            width,
            height,
        }
    }    
}

impl Pluggable for WinitWindowPlugin {
    /// Spawns the main window and triggers the `winit` run loop.
    fn configure(&self, app: &mut App) {
        let event_loop = EventLoop::new()
            .expect("Unable to initialize `Winit` main run loop");

        let title = self.title.clone();
        let width = self.width;
        let height = self.height;

        let window_builder = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height));

        let winit_window: winit::window::Window = window_builder
            .build(&event_loop)
            .expect("Unable to spawn main `Winit` `Window`");

        let raw_window_handle = winit_window.raw_window_handle();
        let raw_display_handle = winit_window.raw_display_handle();

        let host_window = Window::new(
            Box::new(WinitWindowWrapper(winit_window)),
            raw_window_handle,
            raw_display_handle,
        );

        // Add the window as a resource; ensure the `winit_window` is kept alive.
        app.world.add_unique(UniqueWindow {
            host_window: host_window,
        });

        app.set_run_loop(move |app: &mut App| {
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

                    WindowEvent::RedrawRequested => {
                        app.update();
                    }

                    _ => (),
                };
            })
            .expect("Unable to lunch `Winit` event loop");
        });
    }
}