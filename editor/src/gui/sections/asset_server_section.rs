use image::{io::Reader as ImageReader, GenericImageView};

use std::{future::Future, io::Cursor};

use shipyard::{Unique, UniqueView, UniqueViewMut, World};

use engine::{
    egui::{
        ahash::AHashMap, vec2, Grid, Image, Response, Rounding, ScrollArea,
        TextureId, Ui,
    },
    graphics::gpu::AbstractGpu,
    log::{info, warn},
    plugin::graphics::egui::EguiRenderer,
    scene::assets::{asset_server::AssetServer, model::ModelType},
    types::Size,
    wgpu_graphics::{buffer::WGPUTexture, gpu::Gpu},
};

use crate::gui::config::{AssetServerSection, GuiState};

/// A custom asset server used just in the editor just to keep
/// track of each texture as a Egui TextureId to render them
/// in Egui.
#[derive(Unique, Default)]
pub struct EguiAssetServer {
    pub textures: AHashMap<String, TextureId>,
}

/// Syncs the Engine's `AssetStore` to the Editor Egui's `EguiAssetStore`.
pub fn sync_egui_asset_server(world: &World) {
    let gpu = world.borrow::<UniqueView<AbstractGpu>>().unwrap();
    let gpu = gpu.downcast_ref::<Gpu>().unwrap();
    let mut egui_renderer =
        world.borrow::<UniqueViewMut<EguiRenderer>>().unwrap();
    let asset_server = world.borrow::<UniqueView<AssetServer>>().unwrap();
    let mut egui_asset_server =
        world.borrow::<UniqueViewMut<EguiAssetServer>>().unwrap();

    // Take read lock over the asset server.
    let data_lock = asset_server.data.read().unwrap();

    for (id, texture) in &data_lock.textures {
        // Do not sync of the texture is already registered.
        if egui_asset_server.textures.contains_key(id.as_str()) {
            continue;
        }

        let texture = texture.downcast_ref::<WGPUTexture>().unwrap();

        let egui_texture_id = egui_renderer.renderer.register_native_texture(
            &gpu.device,
            &texture.view,
            engine::wgpu::FilterMode::Linear,
        );

        egui_asset_server
            .textures
            .insert((*id).to_owned(), egui_texture_id);
    }
}

pub fn render_asset_server(ui: &mut Ui, world: &World) -> Response {
    let mut gui_state = world.borrow::<UniqueViewMut<GuiState>>().unwrap();

    let gpu = world.borrow::<UniqueView<AbstractGpu>>().unwrap();
    let egui_asset_server =
        world.borrow::<UniqueView<EguiAssetServer>>().unwrap();
    let mut asset_server =
        world.borrow::<UniqueViewMut<AssetServer>>().unwrap();

    let height = ui.available_height();

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.selectable_value(
                &mut gui_state.asset_server.active_asset_server_section,
                Some(AssetServerSection::Texture),
                "Textures",
            );

            ui.selectable_value(
                &mut gui_state.asset_server.active_asset_server_section,
                Some(AssetServerSection::Cubemap),
                "Cubemaps",
            );

            ui.selectable_value(
                &mut gui_state.asset_server.active_asset_server_section,
                Some(AssetServerSection::Mesh),
                "Meshes",
            );
        });

        ui.separator();

        match gui_state.asset_server.active_asset_server_section {
            Some(AssetServerSection::Texture) => render_texture_section(
                ui,
                &mut asset_server,
                &egui_asset_server,
                height,
            ),

            Some(AssetServerSection::Mesh) => {
                render_mesh_section(ui, &gpu, &mut asset_server)
            }

            _ => ui.label("No selected"),
        }
    })
    .response
}

fn render_texture_section(
    ui: &mut Ui,
    asset_server: &mut AssetServer,
    egui_asset_server: &EguiAssetServer,
    height: f32,
) -> Response {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            if ui.button("Load texture").clicked() {
                let task = rfd::AsyncFileDialog::new().pick_files();
                let ctx = ui.ctx().clone();

                let loader = asset_server.loader.clone();

                execute(async move {
                    let files = match task.await {
                        Some(f) => f,
                        _ => return,
                    };

                    // TODO(Angel): Load in parallel.
                    for file in files {
                        let buffer = file.read().await;

                        let raw_img =
                            match ImageReader::new(Cursor::new(buffer))
                                .with_guessed_format()
                            {
                                Ok(i) => i,
                                _ => return,
                            };

                        let img = match raw_img.decode() {
                            Ok(i) => i,
                            _ => return,
                        };

                        let (width, height) = img.dimensions();

                        let buffer = img.to_rgba8().into_raw();
                        let mut loader_lock = loader.lock().unwrap();
                        loader_lock.load_texture(
                            file.file_name(),
                            buffer,
                            Size::new(width, height),
                        );
                    }

                    ctx.request_repaint();
                });
            }
        });

        let width = ui.available_width();

        let (_, rect) = ui.allocate_space(vec2(width, height));
        let mut ui = ui.child_ui(rect, Default::default());

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(&mut ui, |ui| {
                Grid::new("textures_asset_server_grid").show(ui, |ui| {
                    for (id, texture) in &egui_asset_server.textures {
                        ui.push_id(id, |ui| {
                            ui.vertical(|ui| {
                                let image = Image::new((
                                    *texture,
                                    engine::egui::Vec2::new(100.0, 100.0),
                                ))
                                .rounding(Rounding::same(4.0));
                                // Render the scene.
                                ui.add(image);
                                ui.label(id);
                            })
                            .response
                            .context_menu(|ui| {
                                if ui.button("Delete").clicked() {
                                    println!("Delete texture...")
                                }
                            })
                        });
                    }
                })
            });
    })
    .response
}

fn render_mesh_section(
    ui: &mut Ui,
    gpu: &AbstractGpu,
    asset_server: &mut AssetServer,
) -> Response {
    ui.vertical(|ui| {
        if ui.button("Load gltf").clicked() {
            let task = rfd::AsyncFileDialog::new().pick_files();
            let ctx = ui.ctx().clone();

            let loader = asset_server.loader.clone();

            execute(async move {
                let files = match task.await {
                    Some(f) => f,
                    _ => return,
                };

                let mut loader_lock = loader.lock().unwrap();

                for file in files {
                    let models = match ModelType::Obj(file.path()).load_model()
                    {
                        Ok(m) => m,
                        _ => {
                            warn!("Error loading {:?}", file.path());
                            continue;
                        }
                    };

                    models.into_iter().for_each(|m| {
                        loader_lock.load_model(file.file_name(), m)
                    });
                }

                ctx.request_repaint();
            });
        }
    })
    .response
}

fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || engine::futures_lite::future::block_on(f));
}
