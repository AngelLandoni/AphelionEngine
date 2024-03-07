use engine::{
    egui::{
        color_picker::{self, Alpha},
        Context, Label, Slider, Widget, Window,
    },
    plugin::graphics::egui::EguiContext,
};
use shipyard::{UniqueView, UniqueViewMut, World};

use crate::gui::config::{GuiConfig, GuiState};

pub fn render_gizmo_settings(world: &World) {
    let egui = world.borrow::<UniqueView<EguiContext>>().unwrap();
    let mut gui_state = world.borrow::<UniqueViewMut<GuiState>>().unwrap();
    let mut gui_config = world.borrow::<UniqueViewMut<GuiConfig>>().unwrap();

    Window::new("Gizmo settings")
        .resizable(false)
        .open(&mut gui_state.windows.is_gizmo_settings_open)
        .show(&egui.0, |ui| {
            Slider::new(&mut gui_config.gizmo.size, 10.0..=500.0)
                .text("Gizmo size")
                .ui(ui);
            Slider::new(&mut gui_config.gizmo.width, 0.1..=10.0)
                .text("Stroke width")
                .ui(ui);
            Slider::new(&mut gui_config.gizmo.inactive_alpha, 0.0..=1.0)
                .text("Inactive alpha")
                .ui(ui);
            Slider::new(&mut gui_config.gizmo.highlighted_alpha, 0.0..=1.0)
                .text("Highlighted alpha")
                .ui(ui);

            ui.horizontal(|ui| {
                color_picker::color_edit_button_srgba(
                    ui,
                    &mut gui_config.gizmo.x_color,
                    Alpha::Opaque,
                );
                Label::new("X axis color").wrap(false).ui(ui);
            });

            ui.horizontal(|ui| {
                color_picker::color_edit_button_srgba(
                    ui,
                    &mut gui_config.gizmo.y_color,
                    Alpha::Opaque,
                );
                Label::new("Y axis color").wrap(false).ui(ui);
            });

            ui.horizontal(|ui| {
                color_picker::color_edit_button_srgba(
                    ui,
                    &mut gui_config.gizmo.z_color,
                    Alpha::Opaque,
                );
                Label::new("Z axis color").wrap(false).ui(ui);
            });

            ui.horizontal(|ui| {
                color_picker::color_edit_button_srgba(
                    ui,
                    &mut gui_config.gizmo.s_color,
                    Alpha::Opaque,
                );
                Label::new("Screen axis color").wrap(false).ui(ui);
            });
            ui.end_row();
        });
}
