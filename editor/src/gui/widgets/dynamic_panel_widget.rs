use engine::egui::{
    Color32, Frame, Margin, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2
};

use crate::gui::split_panel_tree::{
    BinaryOps, HFraction, PanelNode, SplitPanelTree, Tab, VFraction,
};

use super::tab_widget::render_tab_widget;

const PANEL_CORNER_RADIUS: f32 = 5.0;
const PANEL_SPACE: f32 = 4.0;

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
                        + Vec2::new(rect.width() * left_fraction, 0.0) - Vec2::new(PANEL_SPACE / 2.0, 0.0),
                );

                // Define the size and position of the red rectangle (right half)
                let right_rect = Rect::from_min_max(
                    rect.right_top()
                        - Vec2::new(rect.width() * right_fraction, 0.0) + Vec2::new(PANEL_SPACE / 2.0, 0.0),
                    rect.right_bottom(),
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
                        + Vec2::new(0.0, rect.height() * top_fraction) - Vec2::new(0.0, PANEL_SPACE / 2.0),
                );

                // Define the size and position of the red rectangle (right half)
                let bottom_rect = Rect::from_min_max(
                    rect.left_bottom()
                        - Vec2::new(0.0, rect.height() * bottom_fraction) + Vec2::new(0.0, PANEL_SPACE / 2.0),
                    rect.right_bottom(),
                );

                panel_tree.tree[index.left()].update_rect(top_rect);
                panel_tree.tree[index.right()].update_rect(bottom_rect);
            }

            PanelNode::Container { rect, tabs } => {
                ui.allocate_rect(*rect, Sense::focusable_noninteractive());
                let mut ui = ui.child_ui(*rect, Default::default());

                render_list_of_tabs(&mut ui, rect, tabs, &builder);
            }
        }
    }
}

/// Renders all the tabs and the content of the seleted one.
fn render_list_of_tabs(ui: &mut Ui, rect: &Rect, tabs: &Vec<Tab>, builder: impl Fn(&mut Ui, &Tab) -> Response) {
    // Render the tabs background line.
    ui.painter().rect_filled(
        Rect::from_min_max(
            rect.left_top(),
            rect.right_top() + Vec2::new(0.0, 30.0),
        ), 
        0.0,
        Color32::from_hex("#151515").unwrap_or(Color32::default()),
    );

    for tab in tabs {
        Frame::none()
            .outer_margin(Margin {
                top: 5.0,
                ..Default::default()
            })
            .show(ui, |ui| {
                render_tab_widget(ui, crate::gui::icons::IMAGE, &tab.title, true);
            });
    }

    ui.painter().rect_filled(
        Rect::from_min_max(
            rect.left_top() + Vec2::new(0.0, 30.0),
            rect.right_bottom(),
        ), 
        Rounding {
            nw: 0.0,
            ne: PANEL_CORNER_RADIUS,
            sw: PANEL_CORNER_RADIUS,
            se: PANEL_CORNER_RADIUS,
        },
        Color32::from_hex("#242424").unwrap_or(Color32::default()),
    );

    //builder(ui, tab);
}