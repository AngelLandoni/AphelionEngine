use engine::{
    egui::{Frame, Margin, Response, ScrollArea, TextEdit, Ui},
    scene::hierarchy::{self, Hierarchy},
};
use shipyard::{EntitiesView, EntityId, Get, IntoIter, ViewMut};

use super::hierarchy_widget::HierarchySelectionFlag;

/// Renders the `Properties` section.
pub fn properties_widget(
    ui: &mut Ui,
    entities: &EntitiesView,
    hierarchy: &mut ViewMut<Hierarchy>,
    selection_flag: &mut ViewMut<HierarchySelectionFlag>,
) -> Response {
    ui.vertical(|ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                entities
                    .iter()
                    .filter(|e| selection_flag.get(*e).is_ok())
                    .for_each(|e| {
                        render_section(ui, &e, hierarchy);
                    });
            })
    })
    .response
}

fn render_section(
    ui: &mut Ui,
    entity: &EntityId,
    hierarchy: &mut ViewMut<Hierarchy>,
) {
    Frame::none()
        .inner_margin(Margin::same(20.0))
        .show(ui, |ui| {
            let h = match hierarchy.get(*entity) {
                Ok(h) => h,
                _ => return,
            };

            ui.horizontal_centered(|ui| {
                if ui.button(format!("{}", h.icon)).clicked() {
                    h.title = "Hello!!!".to_owned()
                }

                ui.add_space(4.0);
                let response = ui.add(TextEdit::singleline(&mut h.title));
                if response.changed() {}
            });
        });
}
