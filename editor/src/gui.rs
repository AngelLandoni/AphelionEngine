pub mod split_panel_tree;
pub mod style;
pub mod widgets;

use shipyard::{Unique, UniqueView, UniqueViewMut};
use std::ops::{Deref, DerefMut};

use engine::{
    app::App,
    egui::{Margin, Widget},
    plugin::{graphics::egui::EguiContext, Pluggable},
    schedule::Schedule,
};

use crate::gui::{
    split_panel_tree::{
        HFraction, HSplitDir, SplitPanelTree, VFraction, ROOT_NODE,
    },
    widgets::toolbar_widget::ToolbarWidget,
};

use self::{
    split_panel_tree::VSplitDir, style::configure_fonts,
    widgets::dynamic_panel_widget::render_dynamic_panel_widget,
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
fn configure_gui_system(
    mut panel_state: UniqueViewMut<GuiPanelState>,
    egui: UniqueView<EguiContext>,
) {
    // Configure all the font styles.
    configure_fonts(&egui.0);
    // Configure all the panels.
    configure_panels(&mut panel_state);
}

/// Configures the default layout and distribution of panels within the editor.
fn configure_panels(tree: &mut SplitPanelTree) {
    let (workbench, right_zone) = tree.horizontal_split(
        ROOT_NODE,
        HSplitDir::Left,
        HFraction::Left(0.85),
    );

    let (left_zone, central) = tree.horizontal_split(
        workbench,
        HSplitDir::Right,
        HFraction::Right(0.8),
    );

    let (viewport, logs) =
        tree.vertical_split(central, VSplitDir::Top, VFraction::Top(0.8));

    tree.insert_tab(viewport, "Viewport", "Viewport");
    tree.insert_tab(logs, "General logs", "GeneralLogs");

    tree.insert_tab(left_zone, "Entities", "EntitiesTree");
    tree.insert_tab(right_zone, "Properties", "Properties");
}

/// Renders the UI based on the Panel states.
fn render_gui_system(
    egui: UniqueView<EguiContext>,
    mut panel_state: UniqueViewMut<GuiPanelState>,
) {
    engine::egui::CentralPanel::default()
        .frame(engine::egui::Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            ..Default::default()
        })
        .show(&egui.0, |ui| {
            let rect = ToolbarWidget.ui(ui).rect;

            render_dynamic_panel_widget(
                ui,
                &mut panel_state.0,
                rect.height(),
                |ui, _tab| ui.label("Hello"),
            );
        });
}
