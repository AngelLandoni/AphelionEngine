


use egui::{Context, Visuals, epaint::Shadow};
use egui_winit::State;
use egui_wgpu::{Renderer, renderer::ScreenDescriptor};

use shipyard::{Unique, UniqueView, UniqueViewMut};
use wgpu::{RenderPassDescriptor, RenderPassColorAttachment, Operations};

use crate::{
    app::App,
    plugin::Pluggable,
    schedule::Schedule,
    host::{components::UniqueWindow},
    graphics::{components::{UniqueRenderer, ScreenTexture}, CommandQueue, OrderCommandBuffer, CommandSubmitOrder},
};

use super::window::{WinitWindowWrapper, UniqueWinitEvent};

#[derive(Unique)]
pub struct EguiContext(pub Context);

#[derive(Unique)]
pub struct EguiRenderer {
    state: State,
    renderer: Renderer,
    screen_descriptor: Option<ScreenDescriptor>,
}

pub struct EguiInit();

pub struct EguiPlugin;

// https://github.com/ejb004/egui-wgpu-demo/blob/master/src/gui.rs
impl Pluggable for EguiPlugin {
    fn configure(&self, app: &mut App) {
        {
            let context = Context::default();
            let id = context.viewport_id();

            let visuals = Visuals {
                window_rounding: egui::Rounding::same(2.0),
                window_shadow: Shadow::NONE,

                ..Default::default()
            };

            context.set_visuals(visuals);

            let u_window = app
                .world
                .borrow::<UniqueView<UniqueWindow>>()
                .expect("Unable to adquire Window");

            let gpu = app
                .world
                .borrow::<UniqueView<UniqueRenderer>>()
                .expect("Unable to adquire gpu");

            let state = egui_winit::State::new(
                context.clone(),
                id,
                &u_window.host_window,
                None,
                None
            );

            let renderer = egui_wgpu::renderer::Renderer::new(
                &gpu.gpu.device,
                gpu.gpu.texture_format,
                None,
                1,
            );

            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [
                    gpu.gpu.surface_config.width,
                    gpu.gpu.surface_config.height,
                ],
                pixels_per_point: u_window.host_window.accesor.scale_factor() as f32,
            };

            app.world.add_unique(EguiContext(context));
            
            app.world.add_unique(EguiRenderer {
                state,
                renderer,
                screen_descriptor: Some(screen_descriptor),
            });
        }

        {
            app.schedule(Schedule::BeforeRequestRedraw, |world| {
                world.run(egui_generate_full_output);
            });

            app.schedule(Schedule::AfterRequestRedraw, |world| {
                world.run(egui_render_system);
            });

            app.schedule(Schedule::WindowEvent, |world| {
                world.run(egui_handle_events_system);
            });
        }
    }
}

fn egui_generate_full_output(
    window: UniqueView<UniqueWindow>,
    mut egui: UniqueViewMut<EguiRenderer>,
    egui_ctx: UniqueView<EguiContext>,
) {
    let w = match window.host_window.accesor.downcast_ref::<WinitWindowWrapper>() {
        Some(w) => w,
        None => {
            // TODO(Angel): Use logger.
            println!("Unable to find Winit Window");
            return
        }
    };

    let raw_input = egui.state.take_egui_input(&w.0);
    egui_ctx.0.begin_frame(raw_input);
}

fn egui_render_system(
    gpu: UniqueView<UniqueRenderer>,
    window: UniqueView<UniqueWindow>,
    s_texture: UniqueView<ScreenTexture>,
    queue: UniqueView<CommandQueue>,
    mut egui: UniqueViewMut<EguiRenderer>,
    egui_ctx: UniqueView<EguiContext>,
) {
    let w = match window.host_window.accesor.downcast_ref::<WinitWindowWrapper>() {
        Some(w) => w,
        None => {
            // TODO(Angel): Use logger.
            println!("Unable to find Winit Window");
            return
        }
    };

    let output = egui_ctx.0.end_frame();

    let view = match &s_texture.0 {
        Some(v) => v,
        None => {
            println!("Unable to find screen texture");
            return
        }
    };

    egui.state.handle_platform_output(&w.0, output.platform_output);

    let tris = egui_ctx
        .0
        .tessellate(output.shapes, output.pixels_per_point);
    
    for (id, image_delta) in &output.textures_delta.set {
        egui.renderer.update_texture(
            &gpu.gpu.device,
            &gpu.gpu.queue,
            *id,
            image_delta
        )
    }

    let mut encoder = gpu
        .gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });
    
    
    let s_desc = std::mem::take(&mut egui.screen_descriptor).unwrap();

    egui
        .renderer
        .update_buffers(
            &gpu.gpu.device,
            &gpu.gpu.queue,
            &mut encoder,
            &tris,
            &s_desc,
        );

    {
        let mut r_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Egui render pass"),
            color_attachments: &[
                Some(
                    RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }
                )
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        egui.renderer.render(&mut r_pass, &tris, &s_desc);
    }

    let _ = std::mem::replace(
        &mut egui.screen_descriptor,
        Some(s_desc)
    );

    for t in &output.textures_delta.free {
        egui.renderer.free_texture(t);
    }

    let _ = queue.0.push(OrderCommandBuffer::new(
        Some("Render egui".to_owned()),
        CommandSubmitOrder::DebugGui,
        encoder.finish(),
    ));
}

fn egui_handle_events_system(
    mut egui: UniqueViewMut<EguiRenderer>,
    window: UniqueView<UniqueWindow>,
    winit_event: UniqueView<UniqueWinitEvent>
) {
    let w = match window.host_window.accesor.downcast_ref::<WinitWindowWrapper>() {
        Some(w) => w,
        None => {
            // TODO(Angel): Use logger.
            println!("Unable to find Winit Window");
            return
        }
    };

    let e = match &winit_event.inner {
        Some(e) => e,
        None => {
            println!("Unable to find winit event");
            return
        }
    };

    let _ = egui.state.on_window_event(&w.0, e);
    w.0.request_redraw();
}