use std::process::exit;

use engine::{egui, egui::Widget};

pub struct MenuWidget;

impl Widget for MenuWidget {
    fn ui(self, ui: &mut engine::egui::Ui) -> engine::egui::Response {
        egui::menu::bar(ui, |ui| {
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
