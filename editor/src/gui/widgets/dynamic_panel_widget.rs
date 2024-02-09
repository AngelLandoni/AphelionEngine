use engine::egui::{
    panel, Color32, CursorIcon, Id, LayerId, Order, Pos2, Rect, Response, Rounding, Sense, Ui, Vec2
};
use shipyard::Unique;

use crate::gui::{
    colors::SHADOW_COLOR,
    split_panel_tree::{
        Index,
        BinaryOps, HFraction, PanelNode, SplitPanelTree, Tab, VFraction,
    },
};

use super::{
    shadow_widget::render_shadow_widget, tab_widget::render_tab_widget,
};

const PANEL_CORNER_RADIUS: f32 = 5.0;
const PANEL_SPACE: f32 = 6.0;

#[derive(Unique)]
pub struct TabDragStartPosition(pub Option<Pos2>);

struct HoverData {
    rect: Rect,
    tabs: Option<Rect>,
    dst: Index,
    pointer: Pos2,
}

impl HoverData {
    /*fn resolve(&self) -> (Option<Split>, Rect) {
        if let Some(tabs) = self.tabs {
            return (None, tabs);
        }

        let (rect, pointer) = (self.rect, self.pointer);

        let center = rect.center();
        let pts = [
            center.distance(pointer),
            rect.left_center().distance(pointer),
            rect.right_center().distance(pointer),
            rect.center_top().distance(pointer),
            rect.center_bottom().distance(pointer),
        ];

        let position = pts
            .into_iter()
            .enumerate()
            .min_by(|(_, lhs), (_, rhs)| f32::total_cmp(lhs, rhs))
            .map(|(idx, _)| idx)
            .unwrap();

        let (target, other) = match position {
            0 => (None, Rect::EVERYTHING),
            1 => (Some(Split::Left), Rect::everything_left_of(center.x)),
            2 => (Some(Split::Right), Rect::everything_right_of(center.x)),
            3 => (Some(Split::Above), Rect::everything_above(center.y)),
            4 => (Some(Split::Below), Rect::everything_below(center.y)),
            _ => unreachable!(),
        };

        (target, rect.intersect(other))
    }*/
}

#[derive(Default, Unique)]
pub struct SharedData {
    drag: Option<(Index, usize)>,
    hover: Option<HoverData>,
}

pub fn render_dynamic_panel_widget(
    ui: &mut Ui,
    panel_tree: &mut SplitPanelTree,
    header_height: f32,
    drag_start_position: &mut Option<Pos2>,
    shared_data: &mut SharedData,
    builder: impl Fn(&mut Ui, &Tab) -> Response,
) {
    let full_size = ui.available_size();

    let full_rect = Rect {
        min: Pos2::new(PANEL_SPACE, header_height + PANEL_SPACE),
        max: Pos2::new(
            full_size.x - PANEL_SPACE,
            full_size.y + header_height - PANEL_SPACE,
        ),
    };

    panel_tree.update_root_rect(full_rect);

    ui.allocate_space(full_size);

    // Paint the background.
    ui.painter().rect_filled(
        Rect {
            min: Pos2::new(0.0, header_height),
            max: Pos2::new(full_size.x, full_size.y + header_height),
        },
        0.0,
        Color32::from_hex("#151515").unwrap_or(Color32::default()),
    );

    for index in 0..panel_tree.tree.len() {
        match &mut panel_tree.tree[index] {
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
                        + Vec2::new(rect.width() * left_fraction, 0.0)
                        - Vec2::new(PANEL_SPACE / 2.0, 0.0),
                );

                // Define the size and position of the red rectangle (right half)
                let right_rect = Rect::from_min_max(
                    rect.right_top()
                        - Vec2::new(rect.width() * right_fraction, 0.0)
                        + Vec2::new(PANEL_SPACE / 2.0, 0.0),
                    rect.right_bottom(),
                );

                let separator_rect = Rect::from_min_max(
                    left_rect.right_top(),
                    right_rect.left_bottom(),
                );

                // Resize panel.
                let response = ui
                    .allocate_rect(separator_rect, Sense::click_and_drag())
                    .on_hover_cursor(CursorIcon::ResizeHorizontal);

                {
                    let delta = response.drag_delta().x;
                    let range = rect.max.x - rect.min.x;
                    let min = (PANEL_SPACE / range).min(1.0);
                    let max = 1.0 - min;
                    let (min, max) = (min.min(max), max.max(min));
                    match fraction {
                        HFraction::Left(f) => {
                            *f = (*f + delta / range).clamp(min, max);
                        }
                        HFraction::Right(f) => {
                            *f = (*f + (delta * -1.0) / range).clamp(min, max);
                        }
                    }
                }

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
                        + Vec2::new(0.0, rect.height() * top_fraction)
                        - Vec2::new(0.0, PANEL_SPACE / 2.0),
                );

                // Define the size and position of the red rectangle (right half)
                let bottom_rect = Rect::from_min_max(
                    rect.left_bottom()
                        - Vec2::new(0.0, rect.height() * bottom_fraction)
                        + Vec2::new(0.0, PANEL_SPACE / 2.0),
                    rect.right_bottom(),
                );

                let separator_rect = Rect::from_min_max(
                    top_rect.left_bottom(),
                    bottom_rect.right_top(),
                );

                // Resize panel.
                let response = ui
                    .allocate_rect(separator_rect, Sense::click_and_drag())
                    .on_hover_cursor(CursorIcon::ResizeVertical);

                {
                    let delta = response.drag_delta().y;
                    let range = rect.max.y - rect.min.y;
                    let min = (PANEL_SPACE / range).min(1.0);
                    let max = 1.0 - min;
                    let (min, max) = (min.min(max), max.max(min));
                    match fraction {
                        VFraction::Top(f) => {
                            *f = (*f + delta / range).clamp(min, max);
                        }
                        VFraction::Bottom(f) => {
                            *f = (*f + (delta * -1.0) / range).clamp(min, max);
                        }
                    }
                }

                panel_tree.tree[index.left()].update_rect(top_rect);
                panel_tree.tree[index.right()].update_rect(bottom_rect);
            }

            PanelNode::Container { rect, tabs } => {
                ui.allocate_rect(*rect, Sense::focusable_noninteractive());
                let mut ui = ui.child_ui(*rect, Default::default());

                render_list_of_tabs(&mut ui, rect, tabs, index, drag_start_position, shared_data,&builder);
            }
        }
    }
}

/// Renders all the tabs and the content of the seleted one.
fn render_list_of_tabs(
    ui: &mut Ui,
    rect: &Rect,
    tabs: &Vec<Tab>,
    panel_index: Index,
    drag_start_position: &mut Option<Pos2>,
    shared_data: &mut SharedData,
    _builder: impl Fn(&mut Ui, &Tab) -> Response,
) {
    render_shadow_widget(
        ui,
        Rect::from_min_max(
            rect.left_top() + Vec2::new(0.0, 25.0),
            rect.right_bottom(),
        ),
        SHADOW_COLOR.into(),
        1.0,
        Rounding {
            nw: 0.0,
            ne: PANEL_CORNER_RADIUS,
            sw: PANEL_CORNER_RADIUS,
            se: PANEL_CORNER_RADIUS,
        },
    );

    ui.painter().rect_filled(
        Rect::from_min_max(
            rect.left_top() + Vec2::new(0.0, 25.0),
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

    for (index, tab) in tabs.iter().enumerate() {
        let id = Id::new((panel_index, index, "tab"));
        
        let is_being_dragged = ui.memory(|m| {
            m.is_being_dragged(id)
        });


        if is_being_dragged {
            let layer_id = LayerId::new(Order::Tooltip, id);

            let response = ui.with_layer_id(layer_id, |ui| {
                render_tab_widget(ui, crate::gui::icons::IMAGE, &tab.title, true)
            }).response;

            let sense = Sense::click_and_drag();
            let response = ui
                .interact(response.rect, id, sense)
                .on_hover_cursor(CursorIcon::Grabbing);

            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let center = response.rect.center();
                let start = drag_start_position.unwrap_or(center);

                let delta = pointer_pos - start;
                if delta.x.abs() > 30.0 || delta.y.abs() > 6.0 {
                    ui.ctx().translate_layer(layer_id, delta);

                    shared_data.drag = Some((panel_index, index));
                }
            }

        } else {
            let response = ui.scope(|ui| {
                render_tab_widget(ui, crate::gui::icons::IMAGE, &tab.title, true)
            }).response;
            let sense = Sense::click_and_drag();
            let response = ui.interact(response.rect, id, sense);
            if response.drag_started() {
                *drag_start_position = response.hover_pos();
            }
        }
    }

    //builder(ui, tab);
}
