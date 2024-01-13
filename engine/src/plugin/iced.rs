use std::any::Any;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

use iced::{Font, Pixels, color};
use iced_wgpu::graphics::color;
use iced_wgpu::{
    graphics::Viewport,
    Renderer,
    Backend,
    Settings
};
use iced_winit::runtime::{
    Debug,
    Program,
    program,
};
use shipyard::{UniqueView, Unique, World};

use crate::graphics::gpu::Gpu;
use crate::{
    app::App,
    plugin::Pluggable,
    graphics::components::UniqueRenderer, 
    host::components::UniqueWindow,
};

pub trait AnyIced {
    fn render(&mut self, gpu: &Gpu);
}

pub struct IcedWrapper<T: 'static, C: Program + 'static> {
   viewport: Viewport,
   renderer: Renderer<T>,
   state: program::State<C>,
   debug: Debug,
}

impl<T: 'static, C: Program + 'static> AnyIced for IcedWrapper<T, C> {
    fn render(&mut self, gpu: &Gpu) {
        let screen_frame = gpu.surface.get_current_texture().unwrap();
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let program = self.state.program();

        let view = screen_frame.texture.create_view(
            &wgpu::TextureViewDescriptor::default(),
        );

        println!("Pass render to iced"); 
        
        // And then iced on top
        self.renderer.with_primitives(|backend, primitive| {
            backend.present(
                &gpu.device,
                &gpu.queue,
                &mut encoder,
                Some(color!(0xff00ff)),
                screen_frame.texture.format(),
                &view,
                primitive,
                &self.viewport,
                &self.debug.overlay(),
            );
        });

        gpu.queue.submit(Some(encoder.finish()));
        screen_frame.present();

    }
}

unsafe impl<T: 'static, C: Program + 'static> Send for IcedWrapper<T, C> {}
unsafe impl<T: 'static, C: Program + 'static> Sync for IcedWrapper<T, C> {}

#[derive(Unique)]
pub struct UniqueIced {
    pub(crate) inner: Arc<std::sync::Mutex<Box<dyn AnyIced + Send + Sync>>>
}

pub struct IcedPlugin;

impl Pluggable for IcedPlugin {
    fn configure(&self, app: &mut App) {
        let u_gpu = app
            .world
            .borrow::<UniqueView<UniqueRenderer>>()
            .expect("Unable to adquire GPU");

        let u_window = app
            .world
            .borrow::<UniqueView<UniqueWindow>>()
            .expect("Unable to adquire Window");

        let physical_size = u_window.host_window.inner_size();
        let scale_factor = u_window.host_window.scale_factor();
        
        let viewport = Viewport::with_physical_size(
            iced::Size::new(physical_size.width, physical_size.height),
            scale_factor,
        );

        let mut debug = Debug::new();
        let mut renderer = Renderer::new(
            Backend::new(
                &u_gpu.gpu.device,
                &u_gpu.gpu.queue,
                Settings::default(),
                u_gpu.gpu.texture_format,
            ),
            Font::default(),
            Pixels(16.0),
        );

        let controls = Controls::new();

        let state = program::State::new(
            controls,
            viewport.logical_size(),
            &mut renderer,
            &mut debug,
        );

        let any_iced =  std::sync::Mutex::new(
            Box::new(
                IcedWrapper {
                    viewport,
                    renderer,
                    state,
                    debug,
                } 
            ) as Box<dyn AnyIced + Send + Sync>
        );

        app.world.add_unique(UniqueIced {
            inner: Arc::new(any_iced),
        });
    }
}


use iced_widget::{slider, text_input, Column, Row, Text};
use iced_winit::core::{Alignment, Color, Element, Length};
use iced_winit::runtime::Command;
use iced_winit::style::Theme;

pub struct Controls {
    background_color: Color,
    text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackgroundColorChanged(Color),
    TextChanged(String),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            background_color: Color::BLACK,
            text: String::default(),
        }
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }
}

impl Program for Controls {
    type Renderer = Renderer<Theme>;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::BackgroundColorChanged(color) => {
                self.background_color = color;
            }
            Message::TextChanged(text) => {
                self.text = text;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message, Renderer<Theme>> {
        let background_color = self.background_color;
        let text = &self.text;

        let sliders = Row::new()
            .width(500)
            .spacing(20)
            .push(
                slider(0.0..=1.0, background_color.r, move |r| {
                    Message::BackgroundColorChanged(Color {
                        r,
                        ..background_color
                    })
                })
                .step(0.01),
            )
            .push(
                slider(0.0..=1.0, background_color.g, move |g| {
                    Message::BackgroundColorChanged(Color {
                        g,
                        ..background_color
                    })
                })
                .step(0.01),
            )
            .push(
                slider(0.0..=1.0, background_color.b, move |b| {
                    Message::BackgroundColorChanged(Color {
                        b,
                        ..background_color
                    })
                })
                .step(0.01),
            );

        Row::new()
            .height(Length::Fill)
            .align_items(Alignment::End)
            .push(
                Column::new().align_items(Alignment::End).push(
                    Column::new()
                        .padding(10)
                        .spacing(10)
                        .push(Text::new("Background color").style(Color::WHITE))
                        .push(sliders)
                        .push(
                            Text::new(format!("{background_color:?}"))
                                .size(14)
                                .style(Color::WHITE),
                        )
                        .push(
                            text_input("Placeholder", text)
                                .on_input(Message::TextChanged),
                        ),
                ),
            )
            .into()
    }
}