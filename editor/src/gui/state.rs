use egui_gizmo::GizmoMode;
use shipyard::Unique;

/// Contains the entire state of the GUI, which things are open,
/// what is the state of selection etc.
#[derive(Unique)]
#[derive(Default)]
pub struct GuiState {
    /// Determines if the gizmo tool is selected and which one
    /// it is.
    pub gizmo_type: Option<GizmoMode>,
}


