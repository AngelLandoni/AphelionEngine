use engine::egui::{Context, Response, TopBottomPanel};

pub fn render_top_toolbar_widget(ctx: &Context) -> Response {
    TopBottomPanel::top("top_toolbar")
        .resizable(false)
        .show(ctx, |ui| {})
        .response
}
