use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use shipyard::{Unique, UniqueViewMut};

use winit::{
    dpi::LogicalSize,
    event::{
        DeviceEvent, ElementState, Event, KeyEvent, MouseButton, WindowEvent,
    },
    event_loop::EventLoop,
    keyboard::PhysicalKey,
    platform::macos::WindowBuilderExtMacOS,
    window::WindowBuilder,
};

use crate::{
    app::App,
    host::{
        self,
        events::KeyboardEvent,
        window::{Window, WindowInfoAccessible},
    },
    plugin::Pluggable,
    scene::input::{
        keyboard::KeyCode,
        mouse::{
            CursorDelta, MouseKeyCode, MouseWheelDelta, MouseWheelStepDelta,
        },
    },
    types::Size,
};

pub struct WinitWindowWrapper(pub(crate) winit::window::Window);

impl WindowInfoAccessible for WinitWindowWrapper {
    fn inner_size(&self) -> Size<u32> {
        Size::new(self.0.inner_size().width, self.0.inner_size().height)
    }

    fn scale_factor(&self) -> f64 {
        self.0.scale_factor()
    }
}

#[derive(Unique)]
pub(crate) struct UniqueWinitEvent {
    pub(crate) inner: Option<WindowEvent>,
}

pub struct WinitWindowPlugin {
    title: String,
    size: Size<u32>,
}

impl WinitWindowPlugin {
    /// Creates a new `Winit` Window.
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        WinitWindowPlugin {
            title: title.to_string(),
            size: Size { width, height },
        }
    }
}

impl Pluggable for WinitWindowPlugin {
    /// Spawns the main window and triggers the `winit` run loop.
    fn configure(&self, app: &mut App) {
        let event_loop = EventLoop::new()
            .expect("Unable to initialize `Winit` main run loop");

        let title = self.title.clone();
        let width = self.size.width;
        let height = self.size.height;

        let window_builder = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height))
            .with_title_hidden(true)
            .with_titlebar_transparent(true)
            .with_fullsize_content_view(true);

        let winit_window: winit::window::Window = window_builder
            .build(&event_loop)
            .expect("Unable to spawn main `Winit` `Window`");

        let raw_window_handle = winit_window.raw_window_handle();
        let raw_display_handle = winit_window.raw_display_handle();

        let host_window = Window::new(
            Box::new(WinitWindowWrapper(winit_window)),
            self.size,
            raw_window_handle,
            raw_display_handle,
        );

        // Add the window as a resource; ensure the `winit_window` is kept alive.
        app.world.add_unique(host_window);

        app.world.add_unique(UniqueWinitEvent { inner: None });

        // Winit does not provide a way to know when the delta finished so
        // we need to clean it at the end of each frame.
        app.schedule(crate::schedule::Schedule::EndFrame, |world| {
            let mut cursor_delta = world
                .borrow::<UniqueViewMut<CursorDelta>>()
                .expect("Unable to acquire cursor delta");

            let mut mouse_wheel_delta = world
                .borrow::<UniqueViewMut<MouseWheelDelta>>()
                .expect("Unable to acquire cursor delta");

            let mut mouse_wheel_step_delta = world
                .borrow::<UniqueViewMut<MouseWheelStepDelta>>()
                .expect("Unable to acquire cursor delta");

            cursor_delta.x = 0.0;
            cursor_delta.y = 0.0;

            mouse_wheel_delta.x = 0.0;
            mouse_wheel_delta.y = 0.0;

            mouse_wheel_step_delta.x = 0.0;
            mouse_wheel_step_delta.y = 0.0;
        });

        app.set_run_loop(move |app: &mut App| {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

            event_loop
                .run(move |event, elwt| {
                    elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);

                    // Iced_winit needs the event to behave correctly.
                    match event.clone() {
                        Event::WindowEvent {
                            window_id: _,
                            event,
                        } => {
                            match event {
                                WindowEvent::CloseRequested => elwt.exit(),

                                _ => {}
                            }

                            let mut w_e = app
                                .world
                                .borrow::<UniqueViewMut<UniqueWinitEvent>>()
                                .unwrap();
                            w_e.inner = Some(event);
                        }

                        _ => {}
                    }

                    let host_event = map_winit_events(&event);
                    app.tick(&host_event);
                })
                .expect("Unable to lunch `Winit` event loop");
        });
    }
}

/// Maps the Winit events to host event.
fn map_winit_events<T>(event: &Event<T>) -> host::events::Event {
    match event {
        Event::WindowEvent {
            window_id: _,
            event,
        } => match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => host::events::Event::Keyboard(KeyboardEvent::Pressed(
                map_keyboard_input(key),
            )),

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: key,
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => host::events::Event::Keyboard(KeyboardEvent::Released(
                map_keyboard_input(key),
            )),

            WindowEvent::MouseInput { state, button, .. } => {
                let event = match map_mouse_key_input(button) {
                    Some(e) => e,
                    None => {
                        return host::events::Event::UnknownOrNotImplemented
                    }
                };

                match state {
                    ElementState::Pressed => host::events::Event::Mouse(
                        host::events::MouseEvent::Pressed(event),
                    ),
                    ElementState::Released => host::events::Event::Mouse(
                        host::events::MouseEvent::Released(event),
                    ),
                }
            }

            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    host::events::Event::MouseWheelStepMotion(*x, *y)
                }
                winit::event::MouseScrollDelta::PixelDelta(d) => {
                    host::events::Event::MouseWheelMotion(d.x, d.y)
                }
            },

            WindowEvent::Resized(size) => host::events::Event::Window(
                host::events::WindowEvent::Resized(size.width, size.height),
            ),

            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => host::events::Event::Window(
                host::events::WindowEvent::CursorMoved(position.x, position.y),
            ),

            WindowEvent::CloseRequested => host::events::Event::Window(
                host::events::WindowEvent::CloseRequested,
            ),

            WindowEvent::RedrawRequested => host::events::Event::Window(
                host::events::WindowEvent::RequestRedraw,
            ),

            _ => host::events::Event::Window(
                host::events::WindowEvent::UnknownOrNotImplemented,
            ),
        },

        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } => host::events::Event::CursorMotion(delta.0, delta.1),

        _ => host::events::Event::UnknownOrNotImplemented,
    }
}

/// Contains the range of the position of the letters in the winit environment.
const WINIT_KEYCODE_LETTERS_RANGE: std::ops::Range<u32> = 18..45;

fn map_keyboard_input(key: &PhysicalKey) -> KeyCode {
    match key {
        PhysicalKey::Code(key) => {
            let key_code: u32 = *key as u32;
            // Check if it is a letter.
            if WINIT_KEYCODE_LETTERS_RANGE.contains(&key_code) {
                return KeyCode::from_u32(key_code - 19);
            }

            KeyCode::Unknown
        }
        PhysicalKey::Unidentified(_) => KeyCode::Unknown,
    }
}

fn map_mouse_key_input(button: &MouseButton) -> Option<MouseKeyCode> {
    match button {
        MouseButton::Left => Some(MouseKeyCode::Left),
        MouseButton::Right => Some(MouseKeyCode::Right),
        MouseButton::Middle => Some(MouseKeyCode::Center),
        _ => None,
    }
}
