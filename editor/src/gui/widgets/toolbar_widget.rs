use engine::egui::{
    vec2, Align, Color32, Frame, Layout, Margin, Rounding, Stroke, Style, Vec2,
    Widget,
};

use crate::gui::widgets::menu_widget::MenuWidget;

const MACOS_SEMA_BUTTONS_LEFT_PADDING: f32 = 60.0;
const MACOS_SEMA_BUTTONS_VERTICAL_PADDING: f32 = 6.0;

struct TabWidget<'a> {
    title: &'a str,
}

impl<'a> TabWidget<'a> {
    fn new(title: &'a str) -> Self {
        Self { title }
    }
}

impl<'a> Widget for TabWidget<'a> {
    fn ui(self, ui: &mut engine::egui::Ui) -> engine::egui::Response {
        Frame::group(&Style::default())
            .fill(Color32::from_hex("#242424").unwrap_or_default())
            .stroke(Stroke::NONE)
            .outer_margin(Margin::ZERO)
            .inner_margin(Margin::symmetric(10.0, 5.0))
            .rounding(Rounding {
                nw: 5.0,
                ne: 5.0,
                sw: 0.0,
                se: 0.0,
            })
            .show(ui, |ui| {
                ui.label(self.title);
            })
            .response
    }
}

pub struct ToolbarWidget;
impl Widget for ToolbarWidget {
    fn ui(self, ui: &mut engine::egui::Ui) -> engine::egui::Response {
        ui.allocate_space(vec2(ui.available_width(), 50.0));

        ui.vertical(|ui| {
            // Remove space among items.
            ui.spacing_mut().item_spacing = Vec2::ZERO;

            Frame::group(&Style::default())
                .fill(
                    Color32::from_hex("#151515").unwrap_or_default(),
                )
                .stroke(Stroke::NONE)
                .outer_margin(Margin::ZERO)
                .inner_margin(Margin {
                    left: MACOS_SEMA_BUTTONS_LEFT_PADDING,
                    right: 0.0,
                    top: MACOS_SEMA_BUTTONS_VERTICAL_PADDING,
                    bottom: MACOS_SEMA_BUTTONS_VERTICAL_PADDING,
                })
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 10.0);

                        // Action menu.
                        MenuWidget.ui(ui);

                        // Render all Scene tabs.
                        /*ui.horizontal(|ui| {
                            for i in 0..3 {
                                TabWidget::new(format!("[{}]", i).as_str())
                                    .ui(ui);
                            }
                        });*/
                    });
                });

            Frame::group(&Style::default())
                .fill(
                    Color32::from_hex("#242424").unwrap_or_default(),
                )
                .stroke(Stroke::NONE)
                .outer_margin(Margin::ZERO)
                .inner_margin(Margin::symmetric(20.0, 12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.with_layout(
                            Layout::left_to_right(Align::TOP),
                            |ui| {
                                ui.label("Left");
                            },
                        );

                        ui.with_layout(
                            Layout::centered_and_justified(
                                engine::egui::Direction::TopDown,
                            ),
                            |ui| {
                                ui.label("Play");
                            },
                        );

                        ui.with_layout(
                            Layout::right_to_left(Align::TOP),
                            |ui| {
                                ui.label("Right");
                            },
                        );
                    });
                });
        })
        .response
    }
}
