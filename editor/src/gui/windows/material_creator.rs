use engine::{
    egui::{ahash::AHashMap, Grid, InnerResponse, ScrollArea, Ui, Window},
    graphics::material::{Material, MaterialKind},
    plugin::graphics::egui::EguiContext,
    scene::assets::{asset_server::AssetServer, AssetResourceID},
};
use shipyard::{Unique, UniqueView, UniqueViewMut, World};

use crate::gui::config::GuiState;

#[derive(Unique)]
pub struct MaterialCreatorState {
    kind: MaterialKind,
    name: String,
    textures: AHashMap<String, AssetResourceID>,
}

impl Default for MaterialCreatorState {
    fn default() -> Self {
        MaterialCreatorState {
            kind: MaterialKind::Textured,
            name: "New material".to_string(),
            textures: AHashMap::default(),
        }
    }
}

/// Renders the material creator window.
pub fn render_material_creator(world: &World) {
    let egui = world.borrow::<UniqueView<EguiContext>>().unwrap();
    let mut gui_state = world.borrow::<UniqueViewMut<GuiState>>().unwrap();
    let mut material_creator_state = world
        .borrow::<UniqueViewMut<MaterialCreatorState>>()
        .unwrap();
    let asset_server = world.borrow::<UniqueView<AssetServer>>().unwrap();

    Window::new("Material creator")
        .open(&mut gui_state.windows.is_material_creator_open)
        .show(&egui.0, |ui| {
            Grid::new("material_creator_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Type:");
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut material_creator_state.kind,
                            MaterialKind::Debug,
                            "Debug",
                        );
                        ui.radio_value(
                            &mut material_creator_state.kind,
                            MaterialKind::Textured,
                            "Textured",
                        );
                        ui.radio_value(
                            &mut material_creator_state.kind,
                            MaterialKind::Untextured,
                            "Untextured",
                        );
                    });
                    ui.end_row();

                    ui.label("Name:");
                    ui.text_edit_singleline(&mut material_creator_state.name);
                    ui.end_row();

                    render_textures_rows(
                        ui,
                        &mut material_creator_state,
                        &asset_server,
                    );

                    ui.label("");
                    if ui.button("Create").clicked() {
                        asset_server.register_material(
                            material_creator_state.name.clone(),
                            Material {
                                kind: MaterialKind::Textured,
                                textures: material_creator_state
                                    .textures
                                    .clone(),
                            },
                        );
                    }
                    ui.end_row();
                });
        });
}

fn render_textures_rows(
    ui: &mut Ui,
    state: &mut MaterialCreatorState,
    asset_server: &AssetServer,
) {
    match state.kind {
        MaterialKind::Debug => {}
        MaterialKind::Untextured => {}
        MaterialKind::Textured => {
            ui.label("Diffuse");
            let InnerResponse { inner, response } =
                ui.menu_button("Select texture", |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_height(250.0)
                        .id_source("inspector icons")
                        .show(ui, |ui| {
                            for (id, _) in asset_server.get_textures() {
                                if ui.button(&id).clicked() {
                                    ui.close_menu();
                                    return Some(id);
                                }
                            }

                            None
                        })
                });

            response.on_hover_cursor(engine::egui::CursorIcon::PointingHand);

            if let Some(texture) = inner.and_then(|r| r.inner) {
                state.textures.insert("diffuse".to_owned(), texture);
            }

            ui.end_row();
        }
    }
}
