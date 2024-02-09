use engine::egui::{
    Color32, CursorIcon, FontId, Pos2, Response, Rounding, Sense, Ui, Vec2,
};

use crate::gui::colors::SHADOW_COLOR;

use super::shadow_widget::{render_partial_shadow_widget, ShadowEdge};

const SPACE_BETWEEN_ICON_AND_TITLE: f32 = 10.0;
const HORIZONTAL_PADDING: f32 = 16.0;

const CORNER_RADIUS: f32 = 10.0;

/// Renders a tab widget.
pub fn render_tab_widget(
    ui: &mut Ui,
    icon: char,
    title: &str,
    is_active: bool,
) -> Response {
    let px = ui.ctx().pixels_per_point().recip();

    let font_id = FontId::proportional(15.0);
    let galley =
        ui.painter()
            .layout_no_wrap(title.to_owned(), font_id, Color32::WHITE);

    let icon_color = if is_active {
        Color32::from_hex("#FF853F").unwrap_or_default()
    } else {
        Color32::WHITE
    };

    let icon_font_id = FontId::proportional(18.0);
    let icon_galley = ui.painter().layout_no_wrap(
        format!("{}", icon),
        icon_font_id,
        icon_color,
    );

    let offset = Vec2::new(HORIZONTAL_PADDING, 0.0);
    let text_size = Vec2::new(
        galley.size().x + icon_galley.size().x + SPACE_BETWEEN_ICON_AND_TITLE,
        galley.size().y + icon_galley.size().y,
    );

    let mut desired_size = text_size + offset * 2.0;
    desired_size.y = 25.0;

    let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    if is_active {
        let rounding = Rounding {
            nw: px * CORNER_RADIUS,
            ne: px * CORNER_RADIUS,
            sw: 0.0,
            se: 0.0,
        };

        render_partial_shadow_widget(
            ui,
            rect,
            SHADOW_COLOR.into(),
            1.0,
            Rounding {
                nw: 0.0,
                ne: CORNER_RADIUS,
                sw: CORNER_RADIUS,
                se: CORNER_RADIUS,
            },
            &ShadowEdge {
                left: true,
                right: true,
                top: true,
                bottom: false,
            },
        );

        ui.painter().rect_filled(
            rect,
            rounding,
            Color32::from_hex("#242424").unwrap_or_default(),
        );
    }

    let title_pos = Pos2::new(
        (rect.left_top().x + (desired_size.x - galley.size().x) / 2.0)
            + icon_galley.size().x / 2.0
            + SPACE_BETWEEN_ICON_AND_TITLE / 2.0,
        rect.left_top().y + (desired_size.y - galley.size().y) / 2.0,
    );

    let icon_pos = Pos2::new(
        title_pos.x - icon_galley.size().x - SPACE_BETWEEN_ICON_AND_TITLE,
        rect.left_top().y + (desired_size.y - icon_galley.size().y) / 2.0,
    );

    ui.painter().galley(title_pos, galley, Color32::RED);
    ui.painter().galley(icon_pos, icon_galley, Color32::RED);

    response
}
