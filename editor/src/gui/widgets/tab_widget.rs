use engine::egui::{Align2, Color32, CursorIcon, FontId, Pos2, Response, Sense, Ui, Vec2};

/// Renders a tab widget.
pub fn render_tab_widget(ui: &mut Ui, title: &str, is_active: bool) -> Response {
    let px = ui.ctx().pixels_per_point().recip();

    let font_id = FontId::proportional(14.0);
    let galley = ui
        .painter()
        .layout_no_wrap(title.to_owned(), font_id, Color32::WHITE);

    let offset = Vec2::new(16.0, 0.0);
    let text_size = galley.size();

    let mut desired_size = text_size + offset * 2.0;
    desired_size.y = 25.0;

    let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    if is_active {
        ui.painter().rect_filled(rect, 0.0, Color32::from_hex("#242424").unwrap_or_default());
        
        let mut top_line_rect = rect;
        top_line_rect.max.y -= rect.height() + px * 3.0;
        ui.painter().rect_filled(top_line_rect, 0.0, Color32::RED);
    }

    let pos = Pos2::new(
        rect.left_top().x + (desired_size.x - text_size.x) / 2.0,
        rect.left_top().y + (desired_size.y - text_size.y) / 2.0,
    );

    ui.painter().galley(pos, galley, Color32::RED);

    response
}