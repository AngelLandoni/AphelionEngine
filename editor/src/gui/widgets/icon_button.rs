use engine::egui::{
    pos2, vec2, Align, Align2, Area, Button, Color32, FontFamily, FontId,
    Frame, Galley, Id, Layout, Order, Rect, Response, RichText, Sense, Shape,
    Stroke, Ui,
};

use crate::gui::icons::{Icon, MESH_CUBE};

pub fn render_icon_button(ui: &mut Ui, icon: &Icon, size: f32) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(size, size), Sense::click());

    // Draw background.
    if response.is_pointer_button_down_on() {
        ui.painter()
            .rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 30));
    } else {
        ui.painter()
            .rect_filled(rect, 4.0, Color32::from_rgb(60, 60, 60));
    }
    let icon_size = size * 0.8;
    let padding = (size - icon_size) / 2.0;

    let internal_rect = Rect::from_min_max(
        rect.min + vec2(padding, padding),
        rect.min + vec2(icon_size, icon_size) + vec2(padding, padding),
    );

    icon.as_image().paint_at(ui, internal_rect);

    if response.hovered() {
        ui.painter().rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
        );
    }

    response
}

#[derive(Clone, Copy)]
struct MenuState {
    is_open: bool,
}

impl MenuState {
    fn inverse(&self) -> Self {
        Self {
            is_open: !self.is_open,
        }
    }

    fn closed() -> Self {
        Self { is_open: false }
    }

    fn open() -> Self {
        Self { is_open: true }
    }
}

impl Default for MenuState {
    fn default() -> Self {
        Self { is_open: false }
    }
}

pub fn render_icon_context_button<R>(
    ui: &mut Ui,
    icon: &Icon,
    size: f32,
    content: impl FnOnce(&mut Ui) -> R,
) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(size, size), Sense::click());

    // Draw background.
    /*if response.is_pointer_button_down_on() {
        ui.painter()
            .rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 30));
    } else {
        ui.painter()
            .rect_filled(rect, 4.0, Color32::from_rgb(60, 60, 60));
            }*/

    let mut ui = ui.child_ui(rect, Default::default());

    let image = icon.as_image();
    ui.menu_image_button(image, |ui| {
        content(ui);
    });

    response
}
