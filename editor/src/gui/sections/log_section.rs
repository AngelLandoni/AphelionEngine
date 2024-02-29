use engine::egui::{vec2, Color32, Response, ScrollArea, TextStyle, Ui};

use crate::utils::log::{clean_logs, read_logs};

pub fn render_log_section(ui: &mut Ui) -> Response {
    ui.vertical(|ui| {
        let (_id, toolbar_rect) =
            ui.allocate_space(vec2(ui.available_width(), 30.0));

        let height = ui.available_height();
        let logs = read_logs();

        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);

        ScrollArea::vertical()
            .id_source("log_scroll")
            .auto_shrink(false)
            .max_height(height)
            .stick_to_bottom(true)
            .show_rows(ui, row_height, logs.len(), |ui, range| {
                for row in range {
                    let (level, buffer) = &logs[row];

                    match level {
                        engine::log::Level::Error => {
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    Color32::RED,
                                    format!("[{}]:", level),
                                );
                                ui.add_space(10.0);
                                ui.label(buffer);
                            });
                        }
                        engine::log::Level::Warn => {
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    Color32::YELLOW,
                                    format!("[{}]:", level),
                                );
                                ui.add_space(10.0);
                                ui.label(buffer);
                            });
                        }
                        engine::log::Level::Info => {
                            ui.horizontal(|ui| {
                                ui.label(format!("[{}]:", level));
                                ui.add_space(10.0);
                                ui.label(buffer);
                            });
                        }
                        engine::log::Level::Debug => {
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    Color32::BROWN,
                                    format!("[{}]:", level),
                                );
                                ui.add_space(10.0);
                                ui.label(buffer);
                            });
                        }
                        engine::log::Level::Trace => {
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    Color32::BLUE,
                                    format!("[{}]:", level),
                                );
                                ui.add_space(10.0);
                                ui.label(buffer);
                            });
                        }
                    };
                }
            });

        let mut ui = ui.child_ui(toolbar_rect, Default::default());

        ui.horizontal(|ui| {
            if ui.button("Clean").clicked() {
                clean_logs();
            }
        });
    })
    .response
}
