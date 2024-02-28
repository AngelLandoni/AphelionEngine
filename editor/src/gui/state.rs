use egui_gizmo::{GizmoMode, GizmoOrientation};
use shipyard::Unique;

/// Contains the entire state of the GUI, which things are open,
/// what is the state of selection etc.
#[derive(Unique)]
pub struct GuiState {
    /// Determines if the gizmo tool is selected and which one
    /// it is.
    pub gizmo_type: Option<GizmoMode>,
    /// Determines the orientation of the gizmo (Loca, Global).
    pub gizmo_orientation: GizmoOrientation,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            gizmo_type: None,
            gizmo_orientation: GizmoOrientation::Local,
        }
    }
}


