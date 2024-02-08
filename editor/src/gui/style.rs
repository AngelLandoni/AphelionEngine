use engine::egui::{Context, FontFamily, FontId, TextStyle};

/// Configures all the needed fonts and sizes.
pub fn configure_fonts(ctx: &Context) {
    // Get current context style.
    let mut style = (*ctx.style()).clone();

    // Redefine text_styles
    style.text_styles = [
        (TextStyle::Heading, FontId::new(30.0, FontFamily::Proportional)),
        (
            TextStyle::Name("Heading2".into()),
            FontId::new(25.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Name("Context".into()),
            FontId::new(23.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(15.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(10.0, FontFamily::Proportional)),
    ]
    .into();

    ctx.set_style(style);
}
