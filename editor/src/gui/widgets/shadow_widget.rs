use engine::egui::{Color32, Rect, Rounding, Ui, Vec2};

/// Renders a panel
pub fn render_shadow_widget(
    ui: &mut Ui,
    rect: Rect,
    shadows: Vec<&str>,
    step_size: f32,
    corner_radius: Rounding,
) {
    for (offset, color) in shadows.iter().enumerate().rev() {
        let offset = offset + 1;
        ui.painter().rect_filled(
            Rect::from_min_max(
                rect.left_top()
                    + Vec2::new(
                        -(offset as f32 * step_size),
                        -(offset as f32 * step_size),
                    ),
                rect.right_bottom()
                    + Vec2::new(
                        offset as f32 * step_size,
                        offset as f32 * step_size,
                    ),
            ),
            corner_radius,
            Color32::from_hex(color).unwrap_or_default(),
        );
    }
}

pub struct ShadowEdge {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
}

pub fn render_partial_shadow_widget(
    ui: &mut Ui,
    rect: Rect,
    shadows: Vec<&str>,
    step_size: f32,
    corner_radius: Rounding,
    ignoring: &ShadowEdge,
) {
    for (offset, color) in shadows.iter().enumerate().rev() {
        let left_offset = if ignoring.left { offset + 1 } else { 0 };
        let right_offset = if ignoring.right { offset + 1 } else { 0 };
        let top_offset = if ignoring.top { offset + 1 } else { 0 };
        let bottom_offset = if ignoring.bottom { offset + 1 } else { 0 };

        ui.painter().rect_filled(
            Rect::from_min_max(
                rect.left_top()
                    + Vec2::new(
                        -(left_offset as f32 * step_size),
                        -(top_offset as f32 * step_size),
                    ),
                rect.right_bottom()
                    + Vec2::new(
                        right_offset as f32 * step_size,
                        bottom_offset as f32 * step_size,
                    ),
            ),
            corner_radius,
            Color32::from_hex(color).unwrap_or_default(),
        );
    }
}
