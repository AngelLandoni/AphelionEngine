use egui_gizmo::{GizmoMode, GizmoOrientation};
use engine::{
    egui::{Response, SidePanel},
    log::{error, info, warn},
    plugin::graphics::egui::EguiContext,
};
use shipyard::{UniqueView, UniqueViewMut, World};

use crate::gui::config::GuiState;

/// Renders the leading toolbar widget.
pub fn render_leading_toolbar_widget(world: &World) -> Response {
    let egui = world.borrow::<UniqueView<EguiContext>>().unwrap();
    // Get the gui state in order to modify the active gizmo if it is required.
    let mut gui_state = world.borrow::<UniqueViewMut<GuiState>>().unwrap();

    SidePanel::left("leading_toolbar")
        .resizable(false)
        .max_width(35.0)
        .show(&egui.0, |ui| {
            // Disable the gizmo (enter selection mode).
            if ui.button("Pointer").clicked() {
                gui_state.gizmo.kind = None;
            }

            // Set the translation gizmo.
            if ui.button("Move").clicked() {
                gui_state.gizmo.kind = Some(GizmoMode::Translate);
            }

            // Set the rotate gizmo.
            if ui.button("Rotate").clicked() {
                gui_state.gizmo.kind = Some(GizmoMode::Rotate);
            }

            // Set the scale gizmo.
            if ui.button("Scale").clicked() {
                gui_state.gizmo.kind = Some(GizmoMode::Scale);
            }

            // Set the scale gizmo.
            if ui.button("Local").clicked() {
                gui_state.gizmo.orientation = GizmoOrientation::Local;
            }

            // Set the scale gizmo.
            if ui.button("Global").clicked() {
                gui_state.gizmo.orientation = GizmoOrientation::Global;
            }

            // Set the scale gizmo.
            if ui.button("Log test").clicked() {
                warn!("This is a warning");
                error!("This is an error");
                info!("This is info");
            }
        })
        .response
}
