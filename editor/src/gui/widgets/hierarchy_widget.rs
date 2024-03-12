use engine::{
    egui::{
        vec2, Align2, Color32, FontId, Rect, Response, ScrollArea, Sense, Ui,
    },
    scene::hierarchy::{self, Hierarchy},
};
use shipyard::{
    AddComponent, Component, Delete, EntitiesView, EntityId, Get, Remove,
    ViewMut, World,
};

use crate::gui::{
    colors::HIGHLIGHT,
    icons::{DISCLOSURE_TRI_DOWN, DISCLOSURE_TRI_RIGHT},
};

/// A constant which defines how much space there should be in the
/// left side of the tree item.
const LEFT_OFFSET: f32 = 16.0;
const LEFT_MARGIN: f32 = 5.0;
const ITEM_HEIGHT: f32 = 20.0;
const ICON_SIZE: f32 = 16.0;
const TEXT_SIZE: f32 = 14.0;
const CHEVRON_SIZE: f32 = 18.0;

#[derive(Component)]
pub struct HierarchyDeletionFlag;
#[derive(Component)]
pub struct HierarchySelectionFlag;
#[derive(Component)]
pub struct HierarchyExpandedFlag;

/// Renders a nice hierarcy widget.
pub fn render_hierarchy_widget(ui: &mut Ui, world: &World) -> Response {
    let entities = world.borrow::<EntitiesView>().unwrap();
    let mut hierarchies = world.borrow::<ViewMut<Hierarchy>>().unwrap();
    let mut deletion_flags =
        world.borrow::<ViewMut<HierarchyDeletionFlag>>().unwrap();
    let mut hierarchy_selection =
        world.borrow::<ViewMut<HierarchySelectionFlag>>().unwrap();
    let mut hierarchy_expanded =
        world.borrow::<ViewMut<HierarchyExpandedFlag>>().unwrap();

    let filtered_entities = entities
        .iter()
        .filter_map(|e| hierarchies.get(e).ok().map(|h| (e, h)))
        .filter(|(_, h)| h.level == 0)
        .map(|(e, _)| e)
        .collect::<Vec<_>>();

    ui.vertical(|ui| {
        ui.label("The search part");

        ui.scope(|ui| {
            // Remove any spacing between items.
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

            let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
            scroll.show(ui, |ui| {
                filtered_entities.iter().for_each(|e| {
                    render_item(
                        ui,
                        &e,
                        &mut deletion_flags,
                        &mut hierarchies,
                        &mut hierarchy_selection,
                        &mut hierarchy_expanded,
                    );
                });
            });
        });
    })
    .response
}

/// Renders each of the items, if the entity contains children it gonna also
/// render them.
fn render_item(
    ui: &mut Ui,
    entity: &EntityId,
    deletion_flags: &mut ViewMut<HierarchyDeletionFlag>,
    hierarchies: &ViewMut<Hierarchy>,
    hierarchy_selection: &mut ViewMut<HierarchySelectionFlag>,
    hierarchy_expanded: &mut ViewMut<HierarchyExpandedFlag>,
) {
    // Extract hierarchy information related to the provided entity.
    // We already filtered only entities which contains the component therefore
    // there is not need to be so cautious accessing it.
    let hierarchy = match hierarchies.get(*entity) {
        Ok(h) => h,
        _ => return,
    };

    let level = hierarchy.level;
    let children = &hierarchy.children;
    let title = &hierarchy.title;
    let icon = &hierarchy.icon;

    let full_width = ui.available_width();
    let full_size = vec2(full_width, ITEM_HEIGHT);
    let (rect, response_bg) = ui.allocate_exact_size(full_size, Sense::click());

    let is_selected = hierarchy_selection.get(*entity).is_ok();

    let bg_color = if response_bg.hovered() || is_selected {
        Color32::from_hex(HIGHLIGHT).unwrap_or_default()
    } else {
        Color32::TRANSPARENT
    };

    // Pain the background color of each item.
    ui.painter().rect_filled(rect, 0.0, bg_color);

    // Calculate the left space associated with the item.
    let left_offset = vec2(LEFT_OFFSET * level as f32, 0.0);
    let chevron_pos = rect.left_top()
        + vec2(0.0, ITEM_HEIGHT * 0.5 - CHEVRON_SIZE * 0.5)
        + left_offset
        + vec2(LEFT_MARGIN, 0.0);
    // Calculate the icon position.
    let icon_pos = rect.left_top()
        + vec2(0.0, ITEM_HEIGHT * 0.5 - ICON_SIZE * 0.5)
        + left_offset
        + vec2(CHEVRON_SIZE, 0.0)
        + vec2(LEFT_MARGIN, 0.0);
    // Calculate the position of the title.
    let title_pos = rect.left_center()
        + left_offset
        + vec2(20.0, 0.0)
        + vec2(CHEVRON_SIZE, 0.0)
        + vec2(LEFT_MARGIN, 0.0);

    // Drow chevron.
    let chevron_response_bg = ui.allocate_rect(
        Rect::from_min_size(chevron_pos, vec2(CHEVRON_SIZE, CHEVRON_SIZE)),
        Sense::click(),
    );

    if !children.is_empty() {
        ui.painter().text(
            chevron_pos,
            Align2::LEFT_TOP,
            if hierarchy_expanded.contains(*entity) {
                DISCLOSURE_TRI_DOWN
            } else {
                DISCLOSURE_TRI_RIGHT
            },
            FontId::proportional(CHEVRON_SIZE),
            Color32::WHITE,
        );
    }

    // Draw icon.
    ui.painter().text(
        icon_pos,
        Align2::LEFT_TOP,
        icon,
        FontId::proportional(ICON_SIZE),
        Color32::WHITE,
    );

    // Draw title.
    ui.painter().text(
        title_pos,
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(TEXT_SIZE),
        Color32::WHITE,
    );

    // If the chevron is pressed the list must be expanded.
    if chevron_response_bg.clicked() {
        if hierarchy_expanded.contains(*entity) {
            hierarchy_expanded.remove(*entity);
        } else {
            hierarchy_expanded
                .add_component_unchecked(*entity, HierarchyExpandedFlag);
        }
    }

    if response_bg.clicked() {
        if is_selected {
            hierarchy_selection.delete(*entity);
        } else {
            hierarchy_selection.clear();
            hierarchy_selection
                .add_component_unchecked(*entity, HierarchySelectionFlag);
        }
    }

    response_bg.context_menu(|ui| {
        entity_action_menus(ui, entity, deletion_flags, hierarchies);
    });

    // Recursivelly render each child.
    // Only render the children if there are and if they are visible.
    if hierarchy_expanded.contains(*entity) && !children.is_empty() {
        for e in children {
            render_item(
                ui,
                e,
                deletion_flags,
                hierarchies,
                hierarchy_selection,
                hierarchy_expanded,
            );
        }
    }
}

fn entity_action_menus(
    ui: &mut Ui,
    entity: &EntityId,
    deletion_flags: &mut ViewMut<HierarchyDeletionFlag>,
    hierarchies: &ViewMut<Hierarchy>,
) {
    if ui
        .button(format!("{} Delete", crate::gui::icons::REMOVE))
        .clicked()
    {
        // TODO(Angel): Just testing, we should also add all children and sub children.
        // TODO(Angel): Remove Children from parent in the hierarchy.
        let mut to_mark_as_deleted = vec![*entity];

        let mut i = 0;
        while i < to_mark_as_deleted.len() {
            let entity = to_mark_as_deleted[i];
            let hierarchy = match hierarchies.get(entity) {
                Ok(h) => h,
                _ => return,
            };

            deletion_flags
                .add_component_unchecked(entity, HierarchyDeletionFlag);

            for c in &hierarchy.children {
                to_mark_as_deleted.push(*c);
            }

            i += 1;
        }

        ui.close_menu();
    }

    if ui
        .button(format!("{} Rename", crate::gui::icons::EDITMODE_HLT))
        .clicked()
    {
        ui.close_menu();
    }
}
