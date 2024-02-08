use engine::egui::{Context, FontData, FontDefinitions, FontFamily, FontId, TextStyle};

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
        (
            TextStyle::Name("Icon".into()),
            FontId::new(20.0, FontFamily::Proportional),
        ),
    ]
    .into();

    ctx.set_style(style);
}

/// Configures the icon font used to render icons.
pub fn configure_icon_font(ctx: &mut Context) {
    let font = FontData::from_static(include_bytes!("../assets/fonts/icon.ttf"));

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert("blender".to_owned(), font);
    fonts.families.insert(
        FontFamily::Name("blender".into()),
        vec!["Hack".to_owned(), "blender".into()],
    );
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .push("blender".to_owned());

    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("blender".to_owned());

    ctx.set_fonts(fonts);
}