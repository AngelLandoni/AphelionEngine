use engine::{
    egui::{
        vec2, Align2, Color32, FontId, Id, Response, ScrollArea, Sense, Ui,
    },
    scene::hierarchy::Hierarchy,
};
use shipyard::{
    AddComponent, AllStoragesViewMut, Component, EntitiesView, EntitiesViewMut,
    EntityId, Get, Unique, UniqueView, UniqueViewMut, View, ViewMut, World,
};

/// A constant which defines how much space there should be in the
/// left side of the tree item.
const LEFT_OFFSET: f32 = 16.0;
const ITEM_HEIGHT: f32 = 20.0;

#[derive(Component)]
pub struct HierarchyDeletionFlag;

/// Renders a nice hierarcy widget.
pub fn render_hierarchy_widget(
    ui: &mut Ui,
    entities: &EntitiesView,
    items: &View<Hierarchy>,
    mut deletion_flags: &mut ViewMut<HierarchyDeletionFlag>,
) -> Response {
    ui.vertical(|ui| {
        ui.label("The search part");

        ui.scope(|ui| {
            // Remove any spacing between items.
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

            let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
            scroll.show(ui, |ui| {
                entities
                    .iter()
                    .filter_map(|e| items.get(e).ok().map(|h| (e, h)))
                    // We want onyl the first level, the `render_item` is
                    // in charge of render the children.
                    .filter(|(_, h)| h.level == 0)
                    .for_each(|(e, _)| {
                        render_item(ui, &e, &mut deletion_flags, items);
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
    hierarchies: &View<Hierarchy>,
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

    let full_width = ui.available_width();
    let full_size = vec2(full_width, ITEM_HEIGHT);
    //let (rect, response_bg) = ui.allocate_exact_size(full_size, Sense::hover());
    let (rect, response_bg) = ui.allocate_exact_size(full_size, Sense::click());

    let bg_color = if response_bg.hovered() {
        Color32::DARK_RED
    } else {
        Color32::TRANSPARENT
    };

    // Calculate the left space associated with the item.
    let left_offset = vec2(LEFT_OFFSET * level as f32, 0.0);
    // Calculate the position of the title.
    let title_pos = rect.left_center() + left_offset;

    // Pain the background color of each item.
    ui.painter().rect_filled(response_bg.rect, 0.0, bg_color);
    ui.painter().text(
        title_pos,
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(16.0),
        Color32::WHITE,
    );

    // TODO(Angel): Just testing, we should also add all children and sub children.
    // TODO(Angel): Remove Children from parent in the hierarchy.
    if response_bg.clicked() {
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
    }

    // Recursivelly render each child.
    for e in children {
        render_item(ui, e, deletion_flags, hierarchies);
    }
}
