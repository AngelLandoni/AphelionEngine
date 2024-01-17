use std::sync::Arc;

use winit::keyboard::ModifiersState;
use iced::{Font, Pixels, Theme};
use iced_wgpu::core::renderer;
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
use iced_winit::conversion;

use iced_widget::runtime::Program;
use shipyard::{UniqueView, Unique, UniqueViewMut};

use crate::graphics::CommandQueue;
use crate::graphics::components::{ScreenTexture, ScreenFrame};
use crate::graphics::{OrderCommandQueue, OrderCommandBuffer, CommandSubmitOrder};
use crate::{
    app::App,
    plugin::{
        Pluggable,
        window::UniqueWinitEvent,
    },
    graphics::{
        gpu::Gpu,
        components::UniqueRenderer
    }, 
    host::components::{
        UniqueCursor,
        UniqueWindow
    },
    types::Size,
    schedule::Schedule,
};

pub(crate) trait AnyIced {
    fn pre_frame_config(&mut self, gpu: &Gpu);
    fn render(&mut self, gpu: &Gpu, queue: &OrderCommandQueue, screen_frame: &ScreenFrame, screen_texture: &ScreenTexture);
    fn queue_event(&mut self, event: iced::Event);
    fn update(&mut self, cursor_x: f64, cursor_y: f64);
    fn window_resized(&mut self, size: Size<u32>, scale_factor: f64);
}

//pub struct IcedWrapper<P: iced_widget::runtime::Program + 'static> {
pub struct IcedWrapper<P>
    where
    P: Program<Renderer = Renderer<iced::Theme>> + 'static,
{ 
   viewport: Viewport,
   renderer: Renderer<<P::Renderer as iced_core::Renderer>::Theme>,
   state: program::State<P>,
   debug: Debug,
   theme: <P::Renderer as iced_core::Renderer>::Theme,
   should_redraw: bool,
   should_resize: Option<(Size<u32>, f64)>,
}

impl<P: Program<Renderer = Renderer<iced::Theme>> + 'static> AnyIced for IcedWrapper<P> {
    fn pre_frame_config(&mut self, gpu: &Gpu) {
        if let Some(s_r) = &self.should_resize {
            let size = s_r.0;

            self.viewport = Viewport::with_physical_size(
                iced::Size::new(s_r.0.width, s_r.0.height),
                s_r.1,
            );

            let device = &gpu.device;
            gpu.surface.configure(
                device,
                 &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: gpu.texture_format,
                    width: size.width,
                    height: size.height,
                    present_mode: wgpu::PresentMode::AutoVsync,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
               }
            );

            self.should_resize = None;
        }
    }

    fn render(&mut self, gpu: &Gpu, queue: &OrderCommandQueue, screen_frame: &ScreenFrame, screen_texture: &ScreenTexture) {
        

        if !self.should_redraw { return }
        self.should_redraw = false;

        if let Some(frame) = &screen_frame.0 {
            if let Some(view) = &screen_texture.0 {
                let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                // And then iced on top
                self.renderer.with_primitives(|backend, primitive| {
                    backend.present(
                        &gpu.device,
                        &gpu.queue,
                        &mut encoder,
                        None,
                        frame.texture.format(),
                        &view,
                        primitive,
                        &self.viewport,
                        &self.debug.overlay(),
                    );
                });
        
                let _ = queue.push(OrderCommandBuffer::new(
                    Some("Iced step".to_owned()),
                    CommandSubmitOrder::DebugGui,
                    encoder.finish(),
                ));
            }
        }
    }

    fn queue_event(&mut self, event: iced::Event) {
        self.state.queue_event(event);
    }

    fn update(&mut self, cursor_x: f64, cursor_y: f64) {
        if self.state.is_queue_empty() { return }

        let mut c = Clipboard::unconnected();

        let cursor = iced::mouse::Cursor::Available(conversion::cursor_position(
            winit::dpi::PhysicalPosition::new(cursor_x, cursor_y),
            self.viewport.scale_factor()
        )); 

        let _ = self.state.update(
            self.viewport.logical_size(),
            cursor,
            &mut self.renderer,
            &self.theme,
            &renderer::Style {
                text_color: Color::WHITE,
            },
            &mut c,
            &mut self.debug,
        );

        self.should_redraw = true;
    }

    fn window_resized(&mut self, size: Size<u32>, scale_factor: f64) {
        self.should_resize = Some((size, scale_factor));
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
        {
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
                        theme: Theme::Light,
                        should_redraw: true,
                        should_resize: None,
                    } 
                ) as Box<dyn AnyIced + Send + Sync>
            );

            app.world.add_unique(UniqueIced {
                inner: Arc::new(any_iced),
            });
        }
        
        {
            app.schedule(Schedule::WindowEvent, |world| {
                world.run(iced_update_event_queue_system);
            });

            app.schedule(Schedule::Start, |world| {
                let u_iced = world.borrow::<UniqueView<UniqueIced>>().unwrap();
                let u_gpu = world.borrow::<UniqueView<UniqueRenderer>>().unwrap();
                u_iced.inner.lock().unwrap().pre_frame_config(&u_gpu.gpu);
            });

            app.schedule(Schedule::Update, |world| {
                world.run(iced_render_system);
            });

            app.schedule(Schedule::WindowEvent, |world| {
                // TODO(Angel): Move this to a system.
                let u_iced = world.borrow::<UniqueView<UniqueIced>>().unwrap();
                let u_cursor = world.borrow::<UniqueView<UniqueCursor>>().unwrap();
                u_iced.inner.lock().unwrap().update(u_cursor.x, u_cursor.y);
            });

            app.schedule(Schedule::WindowResize, |world| {
                world.run(resize_window_system);
            });
        }

    }
}

fn iced_update_event_queue_system(u_window: UniqueView<UniqueWindow>,
                                  u_iced: UniqueView<UniqueIced>,
                                  u_winit_event: UniqueView<UniqueWinitEvent>) {
    let modifiers = ModifiersState::default();

    let w_e = match &u_winit_event.inner {
        Some(e) => e,
        None => return,
    };

    // Map window event to iced event
    if let Some(event) = iced_winit::conversion::window_event(
        iced_winit::core::window::Id::MAIN,
        w_e,
        u_window.host_window.scale_factor(),
        modifiers
    ) {
        u_iced.inner.lock().unwrap().queue_event(event);
    }
}

fn iced_render_system(u_gpu: UniqueView<UniqueRenderer>,
                      u_iced: UniqueView<UniqueIced>,
                      queue: UniqueView<CommandQueue>,
                      s_frame: UniqueView<ScreenFrame>,
                      s_texture: UniqueView<ScreenTexture>) {
    u_iced.inner.lock().unwrap().render(
        &u_gpu.gpu,
        &queue.0,
        &s_frame,
        &s_texture
    );
}

fn resize_window_system(u_iced: UniqueViewMut<UniqueIced>,
                        u_w: UniqueView<UniqueWindow>) {
    u_iced.inner.lock().unwrap().window_resized(
        u_w.host_window.size,
        u_w.host_window.accesor.scale_factor()
    );
}

use iced_widget::{slider, text_input};
use iced_winit::core::{Alignment, Color, Element, Length};
use iced_winit::runtime::Command;
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, progress_bar, radio,
    row, scrollable, text, toggler, vertical_rule,
    vertical_space,
};

pub struct Controls {
    theme: Theme,
    background_color: Color,
    text: String,
    slider_value: f32,
    checkbox_value: bool,
    toggler_value: bool,
    input_value: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ThemeType {
    Light,
    Dark,
    Custom,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackgroundColorChanged(Color),
    TextChanged(String),
    ThemeChanged(ThemeType),
    InputChanged(String),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
}

impl Default for Controls {
    fn default() -> Self {
        Self::new()
    }
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            theme: Theme::Dark,
            background_color: Color::BLACK,
            text: String::default(),
            slider_value: 0.0,
            checkbox_value: false,
            toggler_value: false,
            input_value: "".to_owned(),
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
            Message::ThemeChanged(theme) => {
                self.theme = match theme {
                    ThemeType::Light => Theme::Light,
                    ThemeType::Dark => Theme::Dark,
                    ThemeType::Custom => Theme::custom(iced::theme::Palette {
                        background: Color::from_rgb(1.0, 0.9, 1.0),
                        text: Color::BLACK,
                        primary: Color::from_rgb(0.5, 0.5, 0.0),
                        success: Color::from_rgb(0.0, 1.0, 0.0),
                        danger: Color::from_rgb(1.0, 0.0, 0.0),
                    }),
                }
            }
            Message::InputChanged(value) => self.input_value = value,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => self.checkbox_value = value,
            Message::TogglerToggled(value) => self.toggler_value = value,
        }

        Command::none()
    }

    fn view(&self) -> Element<Message, Renderer<Theme>> {
            let choose_theme =
            [ThemeType::Light, ThemeType::Dark, ThemeType::Custom]
                .iter()
                .fold(
                    column![text("Choose a theme:")].spacing(10),
                    |column, theme| {
                        column.push(radio(
                            format!("{theme:?}"),
                            *theme,
                            Some(match self.theme {
                                Theme::Light => ThemeType::Light,
                                Theme::Dark => ThemeType::Dark,
                                Theme::Custom { .. } => ThemeType::Custom,
                            }),
                            Message::ThemeChanged,
                        ))
                    },
                );

        let text_input = text_input("Type something...", &self.input_value)
            .on_input(Message::InputChanged)
            .padding(10)
            .size(20);

        let button = button("Submit")
            .padding(10)
            .on_press(Message::ButtonPressed);

        let slider =
            slider(0.0..=100.0, self.slider_value, Message::SliderChanged);

        let progress_bar = progress_bar(0.0..=100.0, self.slider_value);

        let scrollable = scrollable(
            column!["Scroll me!", vertical_space(800), "You did it!"]
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(100);

        let checkbox = checkbox(
            "Check me!",
            self.checkbox_value,
            Message::CheckboxToggled,
        );

        let toggler = toggler(
            String::from("Toggle me!"),
            self.toggler_value,
            Message::TogglerToggled,
        )
        .width(Length::Shrink)
        .spacing(10);

        let content = column![
            choose_theme,
            horizontal_rule(38),
            row![text_input, button]
                .spacing(10)
                .align_items(Alignment::Center),
            slider,
            progress_bar,
            row![
                scrollable,
                vertical_rule(38),
                column![checkbox, toggler].spacing(20)
            ]
            .spacing(10)
            .height(100)
            .align_items(Alignment::Center),
        ]
        .spacing(20)
        .padding(20)
        .max_width(600);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}