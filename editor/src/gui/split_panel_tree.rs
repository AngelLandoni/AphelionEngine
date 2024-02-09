use std::fmt::Debug;

use engine::egui::Rect;

/// Represents a reference to a node.
pub type Index = usize;

pub trait BinaryOps {
    fn left(&self) -> usize;
    fn right(&self) -> usize;
    fn level(&self) -> usize;
    fn parent(&self) -> usize;
    fn is_left(&self) -> bool;
    fn is_right(&self) -> bool;
    fn children_at(&self, level: usize) -> std::ops::Range<usize>;
    fn children_left(&self, level: usize) -> std::ops::Range<usize>;
    fn children_right(&self, level: usize) -> std::ops::Range<usize>;
}

impl BinaryOps for Index {
    fn left(&self) -> usize {
        self * 2 + 1
    }

    fn right(&self) -> usize {
        self * 2 + 2
    }

    fn level(&self) -> usize {
        if *self == 0 {
            return 0;
        }
        (usize::BITS as usize) - (self + 1).leading_zeros() as usize
    }

    fn parent(&self) -> Index {
        (self - 1) / 2
    }

    fn is_left(&self) -> bool {
        self % 2 != 0
    }

    fn is_right(&self) -> bool {
        self % 2 == 0
    }

    fn children_at(&self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self + 1) * base - 1;
        let e = (self + 2) * base - 1;
        s..e
    }

    fn children_left(&self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self + 1) * base - 1;
        let e = (self + 1) * base + base / 2 - 1;
        s..e
    }

    fn children_right(&self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self + 1) * base + base / 2 - 1;
        let e = (self + 2) * base - 1;
        s..e
    }
}

/// Conatins the index of the root node, it must be always 0.
pub const ROOT_NODE: usize = 0;

#[derive(Clone)]
pub struct Tab {
    pub title: String,
    pub identification: String,
}

impl Debug for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tab").field("title", &self.title).finish()
    }
}

/// Represets the layout or item in the tree.
#[derive(Debug)]
pub enum PanelNode {
    None,
    Container {
        rect: Rect,
        tabs: Vec<Tab>,
        active_tab: Index,
    },
    HLayout {
        rect: Rect,
        fraction: HFraction,
    },
    VLayout {
        rect: Rect,
        fraction: VFraction,
    },
}

impl PanelNode {
    pub fn update_rect(&mut self, new_rect: Rect) {
        match self {
            Self::None => (),
            Self::Container { rect, .. }
            | Self::VLayout { rect, .. }
            | Self::HLayout { rect, .. } => *rect = new_rect,
        }
    }

    pub fn is_container(&self) -> bool {
        match self {
            PanelNode::Container { .. } => true,
            _ => false,
        }
    }

    pub fn append_tab(&mut self, tab: Tab) {
        match self {
            PanelNode::Container { tabs, .. } => {
                tabs.push(tab);
            }
            _ => {}
        }
    }

    pub fn extract_tab(&mut self, index: Index) -> Option<Tab> {
        match self {
            PanelNode::Container { tabs, .. } => {
                if tabs.len() < index || tabs.is_empty() {
                    None
                } else {
                    Some(tabs.remove(index))
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum HFraction {
    Left(f32),
    Right(f32),
}

#[derive(Debug)]
pub enum VFraction {
    Top(f32),
    Bottom(f32),
}

#[derive(Copy, Clone, Debug)]
pub enum HSplitDir {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub enum VSplitDir {
    Top,
    Bottom,
}

/// This data structure represents the state of all panels within the
/// application, akin to a hierarchical tree.
///
/// Each panel is stored within a flat `Vec`, with parent-child relationships
/// tracked via indices. Additionally, the Tree contains size information for
/// each panel, facilitating layout management.
#[derive(Debug)]
pub struct SplitPanelTree {
    /// Contains all the nodes in the
    pub tree: Vec<PanelNode>,
}

impl Default for SplitPanelTree {
    fn default() -> Self {
        Self {
            tree: vec![PanelNode::None],
        }
    }
}

impl SplitPanelTree {
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    pub fn update_root_rect(&mut self, rect: Rect) {
        if let Some(n) = self.tree.first_mut() {
            n.update_rect(rect);
        }
    }

    /// Replaces the target node with an horizontal node and returns the left
    /// and right index of the children.
    pub fn horizontal_split(
        &mut self,
        target: Index,
        dir: HSplitDir,
        fraction: HFraction,
    ) -> (Index, Index) {
        let new_layout = PanelNode::HLayout {
            fraction,
            rect: Rect::NOTHING,
        };

        // Take the ownership of the parent position and inject the new type
        // of layout.
        let old = std::mem::replace(&mut self.tree[target], new_layout);

        // Check if the provided index is out of bounds. If it is, the vector
        // needs to grow. Since indices on the right side tend to be larger, the
        // vector must expand to accommodate them.
        let right_most_element = self
            .tree
            .iter()
            .rposition(|n| !matches!(n, PanelNode::None))
            .unwrap_or(0);

        // We want one more level, that implies create N number of elements to
        // fill that level. We take the current level and add an extra one then
        // the 1 gets shifted that amount of levels to the left (multiply by 2).
        let number_of_elements_to_fill_level =
            1 << (right_most_element.level() + 1);

        // Finally resize if it is appropiated.
        self.tree
            .resize_with(number_of_elements_to_fill_level + 1, || {
                PanelNode::None
            });

        // Check where the old content should go (left or right).
        match dir {
            // Insert old in the left (parent * 2 + 1).
            HSplitDir::Left => {
                self.tree[target.left()] = old;
                self.tree[target.right()] = PanelNode::None;
            }
            // Insert old in the left (parent * 2 + 2).
            HSplitDir::Right => {
                self.tree[target.right()] = old;
                self.tree[target.left()] = PanelNode::None;
            }
        }

        (target.left(), target.right())
    }

    /// Replaces the target node with an vertical node and returns the top
    /// and bottom index of the children.
    pub fn vertical_split(
        &mut self,
        target: Index,
        dir: VSplitDir,
        fraction: VFraction,
    ) -> (Index, Index) {
        let new_layout = PanelNode::VLayout {
            fraction,
            rect: Rect::NOTHING,
        };

        // Take the ownership of the parent position and inject the new type
        // of layout.
        let old = std::mem::replace(&mut self.tree[target], new_layout);

        // Check if the provided index is out of bounds. If it is, the vector
        // needs to grow. Since indices on the right side tend to be larger, the
        // vector must expand to accommodate them.
        let right_most_element = self
            .tree
            .iter()
            .rposition(|n| !matches!(n, PanelNode::None))
            .unwrap_or(0);

        // We want one more level, that implies create N number of elements to
        // fill that level. We take the current level and add an extra one then
        // the 1 gets shifted that amount of levels to the left (multiply by 2).
        let number_of_elements_to_fill_level =
            1 << (right_most_element.level() + 1);

        // Finally resize if it is appropiated.
        self.tree
            .resize_with(number_of_elements_to_fill_level + 1, || {
                PanelNode::None
            });

        println!("Len: {}", number_of_elements_to_fill_level);

        // Check where the old content should go (left or right).
        match dir {
            // Insert old in the left (parent * 2 + 1).
            VSplitDir::Top => {
                self.tree[target.left()] = old;
                self.tree[target.right()] = PanelNode::None;
            }
            // Insert old in the left (parent * 2 + 2).
            VSplitDir::Bottom => {
                self.tree[target.right()] = old;
                self.tree[target.left()] = PanelNode::None;
            }
        }

        (target.left(), target.right())
    }

    pub fn insert_tab(
        &mut self,
        target: Index,
        name: &str,
        identification: &str,
    ) {
        match &mut self.tree[target] {
            // Empty spot we are good insert container.
            PanelNode::None => {
                let element = PanelNode::Container {
                    rect: Rect::NOTHING,
                    tabs: vec![Tab {
                        title: name.to_owned(),
                        identification: identification.to_owned(),
                    }],
                    active_tab: 0,
                };

                self.tree[target] = element;
            }

            PanelNode::Container {
                tabs, active_tab, ..
            } => {
                tabs.push(Tab {
                    title: name.to_owned(),
                    identification: identification.to_owned(),
                });
            }

            // Not allowed.
            PanelNode::HLayout {
                rect: _,
                fraction: _,
            } => {}
            PanelNode::VLayout {
                rect: _,
                fraction: _,
            } => {}
        }
    }

    // Removes all the containers which does not have any tab.
    pub fn clean_containers(&mut self) {
        let container =
            self.tree
                .iter()
                .enumerate()
                .find_map(|(index, node)| match node {
                    PanelNode::Container { tabs, .. } => {
                        if tabs.is_empty() {
                            Some(index)
                        } else {
                            None
                        }
                    }
                    _ => None,
                });

        let container: Index = match container {
            Some(c) => c,
            None => return,
        };

        let parent = container.parent();

        self.tree[parent] = PanelNode::None;
        self.tree[container] = PanelNode::None;

        let mut level = 0;

        if container.is_left() {
            'left_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_right(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.tree.len() {
                        break 'left_end;
                    }
                    self.tree[dst] =
                        std::mem::replace(&mut self.tree[src], PanelNode::None);
                }
                level += 1;
            }
        } else {
            'right_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_left(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.tree.len() {
                        break 'right_end;
                    }
                    self.tree[dst] =
                        std::mem::replace(&mut self.tree[src], PanelNode::None);
                }
                level += 1;
            }
        }
    }
}
