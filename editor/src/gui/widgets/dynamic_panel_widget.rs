use engine::{
    egui::{
        Color32, CursorIcon, Id, LayerId, Order, Pos2, Rect, Response,
        Rounding, Sense, Stroke, Ui, Vec2,
    },
    plugin::graphics::egui::EguiContext,
};
use shipyard::{Unique, UniqueView, UniqueViewMut};

use crate::gui::{
    colors::{HIGHLIGHT, SHADOW_COLOR},
    split_panel_tree::{
        BinaryOps, HFraction, HSplitDir, Index, PanelNode, SplitPanelTree, Tab,
        VFraction, VSplitDir,
    },
    GuiPanelState,
};

use super::{
    shadow_widget::render_shadow_widget, tab_widget::render_tab_widget,
};

const PANEL_CORNER_RADIUS: f32 = 5.0;
const PANEL_SPACE: f32 = 6.0;

#[derive(Unique)]
pub struct TabDragStartPosition(pub Option<Pos2>);

pub enum SplitDir {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug)]
struct HoverData {
    rect: Rect,
    tabs: Option<Rect>,
    dst: Index,
    pointer: Pos2,
}

impl HoverData {
    fn calculate_split_section(&self) -> (Option<SplitDir>, Rect) {
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

        let dir = pts
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(d, _)| d)
            .unwrap_or(0);

        let (target, proposed_rect) = match dir {
            0 => (None, Rect::NOTHING),
            1 => (
                Some(SplitDir::Left),
                Rect::from_min_max(
                    rect.left_top(),
                    rect.left_bottom() + Vec2::new(rect.width() * 0.5, 0.0),
                ),
            ),
            2 => (
                Some(SplitDir::Right),
                Rect::from_min_max(
                    rect.right_top() - Vec2::new(rect.width() * 0.5, 0.0),
                    rect.right_bottom(),
                ),
            ),
            3 => (
                Some(SplitDir::Top),
                Rect::from_min_max(
                    rect.left_top(),
                    rect.right_top() + Vec2::new(0.0, rect.height() * 0.5),
                ),
            ),
            4 => (
                Some(SplitDir::Bottom),
                Rect::from_min_max(
                    rect.left_bottom() - Vec2::new(0.0, rect.height() * 0.5),
                    rect.right_bottom(),
                ),
            ),
            _ => unreachable!(),
        };

        (target, proposed_rect)
    }
}

#[derive(Default, Unique, Debug)]
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

    shared_data.drag = None;
    shared_data.hover = None;

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

            PanelNode::Container {
                rect,
                tabs,
                active_tab,
            } => {
                ui.allocate_rect(*rect, Sense::focusable_noninteractive());

                let mut ui = ui.child_ui(*rect, Default::default());

                render_list_of_tabs(
                    &mut ui,
                    rect,
                    tabs,
                    index,
                    active_tab,
                    drag_start_position,
                    shared_data,
                    &builder,
                );
            }
        }
    }
}

/// Renders all the tabs and the content of the seleted one.
fn render_list_of_tabs(
    ui: &mut Ui,
    rect: &Rect,
    tabs: &[Tab],
    panel_index: Index,
    active_tab: &mut Index,
    drag_start_position: &mut Option<Pos2>,
    shared_data: &mut SharedData,
    ui_builder: impl Fn(&mut Ui, &Tab) -> Response,
) {
    let full_response = ui.allocate_rect(*rect, Sense::hover());

    let tabs_rect = Rect::from_min_max(
        rect.left_top(),
        rect.right_top() + Vec2::new(0.0, 25.0),
    );
    let tabs_zone_response = ui.allocate_rect(tabs_rect, Sense::hover());

    let content_rect = Rect::from_min_max(
        rect.left_top() + Vec2::new(0.0, 25.0),
        rect.right_bottom(),
    );

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

    let mut ui = ui.child_ui(tabs_rect, Default::default());

    ui.spacing_mut().item_spacing = Vec2::ZERO;
    ui.horizontal(|ui| {
        for (index, tab) in tabs.iter().enumerate() {
            let id = Id::new((panel_index, index, "tab"));

            let is_being_dragged = ui.memory(|m| m.is_being_dragged(id));
            if is_being_dragged {
                let layer_id = LayerId::new(Order::Foreground, id);

                let response = ui
                    .with_layer_id(layer_id, |ui| {
                        render_tab_widget(
                            ui,
                            crate::gui::icons::IMAGE,
                            &tab.title,
                            *active_tab == index,
                            true,
                        )
                    })
                    .response;

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

                if response.clicked() {
                    *active_tab = index;
                }
            } else {
                let response = ui
                    .scope(|ui| {
                        render_tab_widget(
                            ui,
                            crate::gui::icons::IMAGE,
                            &tab.title,
                            *active_tab == index,
                            false,
                        )
                    })
                    .response;
                let sense = Sense::click_and_drag();
                let response = ui.interact(response.rect, id, sense);

                if response.drag_started() {
                    *drag_start_position = response.hover_pos();
                }
            }
        }
    });

    let mut ui = ui.child_ui(content_rect, Default::default());

    let is_being_dragged = ui.memory(|m| m.is_anything_being_dragged());

    if is_being_dragged && full_response.hovered() {
        shared_data.hover = ui.input(|i| {
            i.pointer.hover_pos().map(|pointer| HoverData {
                rect: *rect,
                dst: panel_index,
                tabs: tabs_zone_response
                    .hovered()
                    .then_some(tabs_zone_response.rect),
                pointer,
            })
        });
    }

    ui.set_clip_rect(*rect);

    if let Some(tab) = tabs.get(*active_tab) {
        ui_builder(&mut ui, tab);
    }
}

/// Calculates, if the tag is being dragged and there is a target, the panel
/// that must be splited and also the hightlights to indicated with and how the
/// pantel will be splited.
pub fn calculate_tag_dragging_system(
    egui: UniqueView<EguiContext>,
    mut panel_state: UniqueViewMut<GuiPanelState>,
    shared_data: UniqueView<SharedData>,
) {
    if let (Some((src, tab_index)), Some(hover)) =
        (shared_data.drag, &shared_data.hover)
    {
        // Only trigger the drag if we are inside a content.
        if !panel_state.tree[hover.dst].is_container() {
            return;
        }

        let (target, zone) = hover.calculate_split_section();

        let id = Id::new("helper");
        let layer_id = LayerId::new(Order::Foreground, id);
        let painter = egui.0.layer_painter(layer_id);

        painter.rect_stroke(
            zone,
            5.0,
            Stroke::new(3.0, Color32::from_hex(HIGHLIGHT).unwrap_or_default()),
        );

        // If the user did not release the tab ignore the logic.
        if !egui.0.input(|i| i.pointer.any_released()) {
            return;
        }

        // Remove the tab from the old place.
        let tab = match panel_state.tree[src].extract_tab(tab_index) {
            Some(tab) => tab,
            None => return,
        };

        // If we do not have direction it means that the tab was dropped inside
        // the tab section and therefore should be moved
        if let Some(target) = target {
            match target {
                SplitDir::Left => {
                    let (left, _) = panel_state.horizontal_split(
                        hover.dst,
                        HSplitDir::Right,
                        HFraction::Right(0.5),
                    );
                    panel_state.insert_tab(
                        left,
                        &tab.title,
                        &tab.identification,
                    );
                }

                SplitDir::Right => {
                    let (_, right) = panel_state.horizontal_split(
                        hover.dst,
                        HSplitDir::Left,
                        HFraction::Left(0.5),
                    );
                    panel_state.insert_tab(
                        right,
                        &tab.title,
                        &tab.identification,
                    );
                }

                SplitDir::Top => {
                    let (top, _) = panel_state.vertical_split(
                        hover.dst,
                        VSplitDir::Bottom,
                        VFraction::Bottom(0.5),
                    );
                    panel_state.insert_tab(
                        top,
                        &tab.title,
                        &tab.identification,
                    );
                }

                SplitDir::Bottom => {
                    let (_, bottom) = panel_state.vertical_split(
                        hover.dst,
                        VSplitDir::Top,
                        VFraction::Top(0.5),
                    );
                    panel_state.insert_tab(
                        bottom,
                        &tab.title,
                        &tab.identification,
                    );
                }
            };
        } else {
            panel_state.tree[hover.dst].append_tab(tab)
        }

        // Clear all the empty containers.
        panel_state.clean_containers();
    }
}
