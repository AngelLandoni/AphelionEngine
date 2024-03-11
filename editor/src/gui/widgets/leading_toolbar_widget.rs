use std::collections::BTreeMap;

use egui_gizmo::{GizmoMode, GizmoOrientation};
use engine::{
    egui::{
        Button, FontFamily, FontId, Response, RichText, SidePanel, TextStyle,
    },
    log::{error, info, warn},
    plugin::graphics::egui::EguiContext,
};
use shipyard::{UniqueView, UniqueViewMut, World};

use crate::gui::{
    config::GuiState,
    icons::{
        CON_CHILDOF, GIZMO_ROTATE, GIZMO_SCALE, GIZMO_TRANSLATE, MESH_CIRCLE,
        MESH_CUBE, OBJECT_ORIGIN, ORIENTATION_GIMBAL, POINTER,
        RESTRICT_SELECT_OFF,
    },
};

use super::icon_button::render_icon_button;

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
            if render_icon_button(ui, &POINTER, 38.0).clicked() {
                gui_state.gizmo.kind = None;
            }

            // Set the translation gizmo.
            if render_icon_button(ui, &GIZMO_TRANSLATE, 38.0).clicked() {
                gui_state.gizmo.kind = Some(GizmoMode::Translate);
            }

            // Set the rotate gizmo.
            if render_icon_button(ui, &GIZMO_ROTATE, 38.0).clicked() {
                gui_state.gizmo.kind = Some(GizmoMode::Rotate);
            }

            // Set the scale gizmo.
            if render_icon_button(ui, &GIZMO_SCALE, 38.0).clicked() {
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
            };
        })
        .response
}
