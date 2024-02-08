use std::process::exit;

use engine::{egui, egui::Widget};

pub struct MenuWidget;

impl Widget for MenuWidget {
    fn ui(self, ui: &mut engine::egui::Ui) -> engine::egui::Response {
        egui::menu::bar(ui, |ui| {
            ui.spacing_mut().button_padding.x += 3.0;

            let space = ui.spacing().button_padding.x;
            ui.add_space(space * 2.0);

            //style.theme(ui);
            {
                let mut visuals = ui.visuals().clone();
                visuals.widgets.noninteractive.bg_stroke.width = 0.0;
                visuals.widgets.inactive.bg_stroke.width = 0.0;
                visuals.widgets.hovered.bg_stroke.width = 0.0;
                visuals.widgets.active.bg_stroke.width = 0.0;
                visuals.widgets.open.bg_stroke.width = 0.0;

                visuals.popup_shadow = egui::epaint::Shadow::default();

                ui.ctx().set_visuals(visuals);
            }
            ui.menu_button("File", |ui| {
                if ui.button("Exit").clicked() {
                    exit(0);
                }
            });

            ui.menu_button("About", |ui| {
                if ui.button("Angel's editor for cool games").clicked() {
                    ui.close_menu();
                }
            });
        })
        .response
    }
}
