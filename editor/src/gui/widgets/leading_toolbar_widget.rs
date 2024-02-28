use egui_gizmo::{GizmoMode, GizmoOrientation};
use engine::egui::{Context, Response, SidePanel};
use shipyard::{UniqueViewMut, World};

use crate::gui::{icons::MOUSE_LMB, state::GuiState};

/// Renders the leading toolbar widget.
pub fn render_leading_toolbar_widget(ctx: &Context, world: &World) -> Response {
    // Get the gui state in order to modify the active gizmo if it is required.
    let mut gui_state = world.borrow::<UniqueViewMut<GuiState>>().unwrap();

    SidePanel::left("leading_toolbar")
        .resizable(false)
        .max_width(35.0)
        .show(ctx, |ui| {
            // Disable the gizmo (enter selection mode).
            if ui.button("Pointer").clicked() {
                gui_state.gizmo_type = None;
            }

            // Set the translation gizmo.
            if ui.button("Move").clicked() {
                gui_state.gizmo_type = Some(GizmoMode::Translate);
            }

            // Set the rotate gizmo.
            if ui.button("Rotate").clicked() {
                gui_state.gizmo_type = Some(GizmoMode::Rotate);
            }

            // Set the scale gizmo.
            if ui.button("Scale").clicked() {
                gui_state.gizmo_type = Some(GizmoMode::Scale);
            }

            // Set the scale gizmo.
            if ui.button("Local").clicked() {
                gui_state.gizmo_orientation = GizmoOrientation::Local;
            }

            // Set the scale gizmo.
            if ui.button("Global").clicked() {
                gui_state.gizmo_orientation = GizmoOrientation::Global;
            }
        })
        .response
}
