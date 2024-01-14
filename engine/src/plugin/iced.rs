use std::any::Any;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

use iced::{Font, Pixels, color, Theme};
use iced_wgpu::core::renderer;
use iced_wgpu::graphics::color;
use iced_wgpu::{
    graphics::Viewport,
    Renderer,
    Backend,
    Settings
};
use iced_winit::Clipboard;
use iced_winit::runtime::{
    Debug,
    program,
};

use iced_widget::runtime::Program;
use shipyard::{UniqueView, Unique, };

use crate::graphics::gpu::Gpu;
use crate::{
    app::App,
    plugin::Pluggable,
    graphics::components::UniqueRenderer, 
    host::components::UniqueWindow,
};

pub trait AnyIced {
    fn render(&mut self, gpu: &Gpu);
    fn queue_event(&mut self, event: iced::Event);
    fn update(&mut self);
}

//pub struct IcedWrapper<P: iced_widget::runtime::Program + 'static> {
pub struct IcedWrapper<P>
    where
    P: Program<Renderer = Renderer<iced::Theme>> + 'static,
{ 
   viewport: Viewport,
   //renderer: Renderer<<C::Renderer as iced_core::Renderer>::Theme>,
   renderer: Renderer<<P::Renderer as iced_core::Renderer>::Theme>,
   state: program::State<P>,
   debug: Debug,
   theme: <P::Renderer as iced_core::Renderer>::Theme,
}

impl<P: Program<Renderer = Renderer<iced::Theme>> + 'static> AnyIced for IcedWrapper<P> {
//impl<R: iced_core::Renderer, P: Program + 'static + <Renderer = R>> AnyIced for IcedWrapper<R, P> {
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
                None,
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

    fn queue_event(&mut self, event: iced::Event) {
        self.state.queue_event(event);
    }

    fn update(&mut self) {
        let mut c = Clipboard::unconnected();

        let _ = self.state.update(
            self.viewport.logical_size(),
            iced::mouse::Cursor::Unavailable,
            &mut self.renderer,
            &self.theme,
            &renderer::Style {
                text_color: Color::WHITE,
            },
            &mut c,
            &mut self.debug,
        );
    }
}

unsafe impl<P: Program<Renderer = Renderer<iced::Theme>> + 'static> Send for IcedWrapper<P> {}
unsafe impl<P: Program<Renderer = Renderer<iced::Theme>> + 'static> Sync for IcedWrapper<P> {}

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
                    theme: Theme::Dark,
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