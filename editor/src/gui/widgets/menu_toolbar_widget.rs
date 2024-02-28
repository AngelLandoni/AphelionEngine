use std::process::exit;

use engine::egui::{vec2, Context, Response, TopBottomPanel};
use shipyard::{UniqueViewMut, World};

use crate::gui::config::GuiState;

pub fn render_menu_toolbar_widget(ctx: &Context, world: &World) -> Response {
    let mut gui_sate = world.borrow::<UniqueViewMut<GuiState>>().unwrap();

    TopBottomPanel::top("menu_toolbar")
        .resizable(false)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // take full width and fixed height:
                let height = ui.spacing().interact_size.y;
                ui.set_min_size(vec2(ui.available_width(), height));
                // Move the menu due to trafict lights.
                ui.add_space(80.0);

                ui.menu_button("File", |ui| {
                    ui.menu_button("Settings", |ui| {
                        if ui.button("Gizmos").clicked() {
                            let state: &mut bool =
                                &mut gui_sate.windows.is_gizmo_settings_open;

                            *state ^= true;
                            ui.close_menu();
                        }
                    });

                    if ui.button("Exit").clicked() {
                        exit(0);
                    }
                });

                ui.menu_button("About", |ui| {
                    if ui.button("angel's editor for cool games").clicked() {
                        ui.close_menu();
                    }
                });
            })
        })
        .response
}
