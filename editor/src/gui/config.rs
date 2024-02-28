use egui_gizmo::{GizmoMode, GizmoOrientation};
use engine::egui::Color32;
use shipyard::Unique;

#[derive(Clone, Copy)]
pub struct GizmoState {
    /// Determines if the gizmo tool is selected and which one
    /// it is.
    pub kind: Option<GizmoMode>,
    /// Determines the orientation of the gizmo (Loca, Global).
    pub orientation: GizmoOrientation,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            kind: None,
            orientation: GizmoOrientation::Local,
        }
    }
}

/// Contains the entire state of the GUI, which things are open,
/// what is the state of selection etc.
#[derive(Unique, Default)]
pub struct GuiState {
    pub gizmo: GizmoState,
}

#[derive(Copy, Clone)]
pub struct GizmoConfig {
    /// Determines the size of the gizmo.
    pub size: f32,
    /// Determines the thickness of the gizmo.
    pub width: f32,
    /// Determines the alpha when the gizmo is inactive (not clicked).
    pub inactive_alpha: f32,
    /// Determines the alpha when the gizmo is active.
    pub highlighted_alpha: f32,
    /// Determines the color of each axis.
    pub x_color: Color32,
    pub y_color: Color32,
    pub z_color: Color32,
    pub s_color: Color32,
}

impl Default for GizmoConfig {
    fn default() -> Self {
        Self {
            size: 125.0,
            width: 4.0,
            inactive_alpha: 0.5,
            highlighted_alpha: 0.9,
            x_color: Color32::from_rgb(255, 0, 148),
            y_color: Color32::from_rgb(148, 255, 0),
            z_color: Color32::from_rgb(0, 148, 255),
            s_color: Color32::WHITE,
        }
    }
}

/// Contains the entire state of the GUI, which things are open,
/// what is the state of selection etc.
#[derive(Copy, Clone, Unique, Default)]
pub struct GuiConfig {
    pub gizmo: GizmoConfig,
}
