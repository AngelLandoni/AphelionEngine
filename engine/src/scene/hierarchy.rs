use shipyard::{Component, EntityId, Get, IntoIter, View, ViewMut};

#[derive(Component)]
pub struct Hierarchy {
    /// Contains the level assigned to the entity.
    pub level: u32,
    /// Contains teh title of the entity realted with this
    /// hierarchy.
    pub title: String,
    /// Contains the parent assocaited to the entity.
    pub parent: Option<EntityId>,
    /// Contains the children associated with the entity.
    pub children: Vec<EntityId>,
}

impl Hierarchy {
    /// Creates and returns a new `Hierarchy` component without children.
    pub fn empty_root(title: String) -> Self {
        Self {
            level: 0,
            title,
            parent: None,
            children: vec![],
        }
    }

    pub fn attached_to(parent: EntityId, title: String) -> Self {
        Self {
            level: 0,
            title,
            parent: Some(parent),
            children: vec![],
        }
    }

    pub fn new(title: String) -> Self {
        Self {
            level: 0,
            title,
            parent: None,
            children: vec![],
        }
    }
}

/// Adds a child to a specific entity. When an entity is added the children
/// must be resync to set the correct level in order to be translated
/// correctly.
// TODO(Angel): Check if it is really necessary to iterate over all the children
// just only the one that it is added shoudl be enough.
pub fn add_child(
    parent: EntityId,
    child: EntityId,
    hierarchy: &mut ViewMut<Hierarchy>,
) {
    let next_level = {
        let h = match hierarchy.get(parent) {
            Ok(h) => h,
            _ => return,
        };

        // Avoid add the child if it is already a children.
        if h.children.contains(&child) {
            return;
        }

        // Set the child and sync the other children.
        h.children.push(child);
        h.level + 1
    };

    // Update the children's parent.
    {
        let h = match hierarchy.get(child) {
            Ok(h) => h,
            _ => return,
        };

        h.level = next_level;
        h.parent = Some(parent);
    }

    // Sync all the children so all of them have the same level.
    sync_children_level(parent, hierarchy);
}

/// Syncs the root entity with the parent.
pub(crate) fn sync_children_level(
    root: EntityId,
    hierarchy: &mut ViewMut<Hierarchy>,
) {
    let (next_level, children) = {
        // Extract the component
        let h = match hierarchy.get(root) {
            Ok(h) => h,
            _ => return,
        };

        // Calculates the level to be assigend to the children.
        (h.level + 1, h.children.clone())
    };

    // Update every children, if the child contains more children iterate
    // over them too.
    for child in children {
        let c_h = match hierarchy.get(child) {
            Ok(h) => h,
            _ => continue,
        };

        c_h.level = next_level;

        if !c_h.children.is_empty() {
            sync_children_level(child, hierarchy);
        }
    }
}
