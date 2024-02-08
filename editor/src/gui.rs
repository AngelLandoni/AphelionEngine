pub mod dynamic_panel_widget;
pub mod menu_widget;
pub mod split_panel_tree;
pub mod toolbar_widget;

use std::ops::{Deref, DerefMut};

use engine::{
    app::App,
    egui::{FontFamily, FontId, Margin, TextStyle, Widget},
    plugin::{core::clock::Clock, graphics::egui::EguiContext, Pluggable},
    schedule::Schedule,
};
use shipyard::{Unique, UniqueView, UniqueViewMut};

use crate::gui::split_panel_tree::VSplitDir;

use self::{
    dynamic_panel_widget::DynamicPanelWidget,
    split_panel_tree::{HFraction, HSplitDir, SplitPanelTree, VFraction, ROOT_NODE},
    toolbar_widget::ToolbarWidget,
};

#[derive(Unique)]
pub struct GuiPanelState(SplitPanelTree);

impl Deref for GuiPanelState {
    type Target = SplitPanelTree;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GuiPanelState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

enum TabSection {
    Workbench,
    Log,
}

pub struct GuiPlugin;
impl Pluggable for GuiPlugin {
    fn configure(&self, app: &mut App) {
        app.world
            .add_unique(GuiPanelState(SplitPanelTree::default()));

        app.world.run(configure_gui_system);

        app.schedule(Schedule::RequestRedraw, |world| {
            world.run(render_gui_system);
        });
    }
}

/// Configures the ui state.
fn configure_gui_system(mut panel_state: UniqueViewMut<GuiPanelState>) {
    let (workbench, _helpers) =
        panel_state.horizontal_split(ROOT_NODE, HSplitDir::Left, HFraction::Left(0.8));
    let (visual_area, _logs) =
        panel_state.vertical_split(workbench, VSplitDir::Top, VFraction::Top(0.8));

    panel_state.insert_tab(visual_area, "Testing", "Workbench");
}

fn render_gui_system(
    egui: UniqueView<EguiContext>,
    mut panel_state: UniqueViewMut<GuiPanelState>,
    clock: UniqueView<Clock>,
) {
    // Get current context style
    let mut style = (*egui.0.style()).clone();

    // Redefine text_styles
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(30.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Name("Heading2".into()),
            FontId::new(25.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Name("Context".into()),
            FontId::new(23.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(15.0, FontFamily::Proportional)),
        (
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(10.0, FontFamily::Proportional),
        ),
    ]
    .into();

    egui.0.set_style(style);

    engine::egui::CentralPanel::default()
        .frame(engine::egui::Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            ..Default::default()
        })
        .show(&egui.0, |ui| {
            let rect = ToolbarWidget.ui(ui).rect;
            DynamicPanelWidget::new(&mut panel_state, rect.height(), |ui, tab| {
                match tab.identification.as_ref() {
                    "Workbench" => ui.label(format!("DT: {}", clock.delta_seconds())),

                    _ => ui.label(""),
                }
            })
            .ui(ui);
        });
}
