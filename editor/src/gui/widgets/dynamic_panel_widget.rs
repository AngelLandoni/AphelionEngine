use engine::egui::{
    Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget,
};

use crate::gui::split_panel_tree::{
    BinaryOps, HFraction, PanelNode, SplitPanelTree, Tab, VFraction,
};

pub fn render_dynamic_panel_widget(
    ui: &mut Ui,
    panel_tree: &mut SplitPanelTree,
    header_height: f32,
    builder: impl Fn(&mut Ui, &Tab) -> Response,
) {
    let full_size = ui.available_size();

    panel_tree.update_root_rect(Rect {
        min: Pos2::new(0.0, header_height),
        max: Pos2::new(full_size.x, full_size.y + header_height),
    });

    ui.allocate_space(full_size);

    for index in 0..panel_tree.tree.len() {
        match &panel_tree.tree[index] {
            PanelNode::None => {}

            PanelNode::HLayout { fraction, rect } => {
                let left_fraction = match fraction {
                    HFraction::Left(f) => *f,
                    HFraction::Right(f) => 1.0 - *f,
                };

                let right_fraction = match fraction {
                    HFraction::Left(f) => 1.0 - *f,
                    HFraction::Right(f) => *f,
                };

                // Define the size and position of the blue rectangle (left half)
                let left_rect = Rect::from_min_max(
                    rect.left_top(),
                    rect.left_bottom()
                        + Vec2::new(rect.width() * left_fraction, 0.0),
                );

                // Define the size and position of the red rectangle (right half)
                let right_rect = Rect::from_min_max(
                    rect.right_top()
                        - Vec2::new(rect.width() * right_fraction, 0.0),
                    rect.right_bottom(),
                );

                ui.painter().rect(
                    left_rect,
                    0.0,
                    Color32::RED,
                    //                        Color32::from_hex("#242424").unwrap_or_default(),
                    Stroke::NONE,
                );

                ui.painter().rect(
                    right_rect,
                    0.0,
                    Color32::BLUE,
                    //Color32::from_hex("#121212").unwrap_or_default(),
                    Stroke::NONE,
                );

                panel_tree.tree[index.left()].update_rect(left_rect);
                panel_tree.tree[index.right()].update_rect(right_rect);
            }

            PanelNode::VLayout { fraction, rect } => {
                let top_fraction = match fraction {
                    VFraction::Top(f) => *f,
                    VFraction::Bottom(f) => 1.0 - *f,
                };

                let bottom_fraction = match fraction {
                    VFraction::Top(f) => 1.0 - *f,
                    VFraction::Bottom(f) => *f,
                };

                // Define the size and position of the blue rectangle (left half)
                let top_rect = Rect::from_min_max(
                    rect.left_top(),
                    rect.right_top()
                        + Vec2::new(0.0, rect.height() * top_fraction),
                );

                // Define the size and position of the red rectangle (right half)
                let bottom_rect = Rect::from_min_max(
                    rect.left_bottom()
                        - Vec2::new(0.0, rect.height() * bottom_fraction),
                    rect.right_bottom(),
                );

                ui.painter().rect(
                    top_rect,
                    0.0,
                    Color32::from_hex("#346434").unwrap_or_default(),
                    //Color32::from_hex("#242424").unwrap_or_default(),
                    Stroke::NONE,
                );

                ui.painter().rect(
                    bottom_rect,
                    0.0,
                    Color32::from_hex("#804080").unwrap_or_default(),
                    //Color32::from_hex("#121212").unwrap_or_default(),
                    Stroke::NONE,
                );

                panel_tree.tree[index.left()].update_rect(top_rect);
                panel_tree.tree[index.right()].update_rect(bottom_rect);
            }

            PanelNode::Container { rect, tabs } => {
                ui.allocate_rect(*rect, Sense::focusable_noninteractive());

                let mut ui = ui.child_ui(*rect, Default::default());

                for tab in tabs {
                    builder(&mut ui, tab);
                }
            }
        }
    }
}
