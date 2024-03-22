use crate::gui::config::{AssetServerSection, GuiState};
use engine::{
    egui::{
        ahash::AHashMap, vec2, Grid, Image, Response, Rounding, ScrollArea,
        Sense, TextureId, Ui,
    },
    graphics::{
        buffer::WGPUTexture,
        gpu::{AbstractGpu, Gpu},
    },
    log::warn,
    plugin::graphics::egui::EguiRenderer,
    scene::assets::{asset_server::AssetServer, model::ModelType},
    types::Size,
};
use image::{io::Reader as ImageReader, GenericImageView};
use shipyard::{Unique, UniqueView, UniqueViewMut, World};
use std::{future::Future, io::Cursor};

/// A custom asset server used just in the editor to keep
/// track of each texture as an Egui TextureId to render them
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

    asset_server
        .get_textures()
        .iter()
        .for_each(|(id, texture)| {
            // Do not sync if the texture is already registered.
            if egui_asset_server.textures.contains_key(id.as_str()) {
                return;
            }

            let texture = texture.downcast_ref::<WGPUTexture>().unwrap();

            let egui_texture_id =
                egui_renderer.renderer.register_native_texture(
                    &gpu.device,
                    &texture.view,
                    engine::wgpu::FilterMode::Linear,
                );

            egui_asset_server
                .textures
                .insert((*id).to_owned(), egui_texture_id);
        });
}

/// Renders the asset server UI.
pub fn render_asset_server(ui: &mut Ui, world: &World) -> Response {
    let mut gui_state = world.borrow::<UniqueViewMut<GuiState>>().unwrap();

    let _gpu = world.borrow::<UniqueView<AbstractGpu>>().unwrap();
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

            ui.selectable_value(
                &mut gui_state.asset_server.active_asset_server_section,
                Some(AssetServerSection::Material),
                "Materials",
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
                render_mesh_section(ui, &mut asset_server, height)
            }

            Some(AssetServerSection::Material) => {
                render_materials_section(ui, &mut asset_server, height)
            }

            _ => ui.label("No selection"),
        }
    })
    .response
}

/// Renders the texture section of the asset server UI.
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

                    // Load each file in parallel.
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

                        loader
                            .write()
                            .expect("Unable to acquire lock")
                            .load_texture(
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

/// Renders the mesh section of the asset server UI.
fn render_mesh_section(
    ui: &mut Ui,
    asset_server: &mut AssetServer,
    height: f32,
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

                let mut loader_lock =
                    loader.write().expect("Unable to acquire lock");

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
                        loader_lock.load_model(
                            format!("{}#{}", file.file_name(), m.name),
                            m,
                        )
                    });
                }

                ctx.request_repaint();
            });
        }

        let width = ui.available_width();

        let (_, rect) = ui.allocate_space(vec2(width, height));
        let mut ui = ui.child_ui(rect, Default::default());

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(&mut ui, |ui| {
                Grid::new("meshes_asset_server_grid").show(ui, |ui| {
                    for mesh in &asset_server.meshes() {
                        ui.push_id(mesh.clone(), |ui| {
                            ui.vertical(|ui| {
                                ui.allocate_exact_size(
                                    vec2(60.0, 60.0),
                                    Sense::click(),
                                );
                                ui.label(mesh.clone());
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

fn render_materials_section(
    ui: &mut Ui,
    _asset_server: &mut AssetServer,
    height: f32,
) -> Response {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.button("Create Material");
        });

        let width = ui.available_width();

        let (_, rect) = ui.allocate_space(vec2(width, height));
        let mut ui = ui.child_ui(rect, Default::default());

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(&mut ui, |ui| {
                Grid::new("material_asset_server_grid").show(ui, |ui| {
                    ui.label("Matsss!!");
                });
            });
    })
    .response
}

/// Executes a future in a separate thread.
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || engine::futures_lite::future::block_on(f));
}
