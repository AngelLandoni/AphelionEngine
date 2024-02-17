pub mod colors;
pub mod icons;
pub mod split_panel_tree;
pub mod style;
pub mod widgets;

use shipyard::{AllStoragesViewMut, Unique, UniqueView, UniqueViewMut, World};
use std::ops::{Deref, DerefMut};

use engine::{
    app::App, egui::{pos2, Color32, Image, Margin, Rect, Response, Rounding, Stroke, TextureId, Ui, Widget}, graphics::gpu::AbstractGpu, plugin::{graphics::egui::{self, EguiContext, EguiRenderer}, Pluggable}, scene::scene_state::SceneState, schedule::Schedule, wgpu_graphics::{buffer::WGPUTexture, gpu::Gpu}
};

use crate::gui::{
    split_panel_tree::{
        HFraction, HSplitDir, SplitPanelTree, VFraction, ROOT_NODE,
    },
    widgets::toolbar_widget::ToolbarWidget,
};

use self::{
    split_panel_tree::VSplitDir,
    style::{configure_fonts, configure_icon_font},
    widgets::dynamic_panel_widget::{
        calculate_tag_dragging_system, render_dynamic_panel_widget, SharedData,
        TabDragStartPosition,
    },
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

#[derive(Unique)]
pub struct GuiResources {
    workbench_texture_id: TextureId,
}

pub struct GuiPlugin;
impl Pluggable for GuiPlugin {
    fn configure(&self, app: &mut App) {
        app.world
            .add_unique(GuiPanelState(SplitPanelTree::default()));
        app.world.add_unique(TabDragStartPosition(None));
        app.world.add_unique(SharedData::default());

        app.world.run(configure_gui_system);

        app.schedule(Schedule::BeforeStart, |world| {
            register_workbench_texture(&world);
        });

        app.schedule(Schedule::RequestRedraw, |world| {
            world.run(render_gui_system);
            world.run(calculate_tag_dragging_system);
        });
    }
}

/// Configures the ui state.
fn configure_gui_system(
    mut panel_state: UniqueViewMut<GuiPanelState>,
    mut egui: UniqueViewMut<EguiContext>,
) {
    // Configure all the font styles.
    configure_fonts(&egui.0);
    // Configure the icons font.
    configure_icon_font(&mut egui.0);
    // Configure all the panels.
    configure_panels(&mut panel_state);
}

// TODO(Angel): Use AllStorageView
fn register_workbench_texture(world: &World) {
    let mut egui_renderer = world.borrow::<UniqueViewMut<EguiRenderer>>().unwrap();
    let s_state = world.borrow::<UniqueView<SceneState>>().unwrap();
    let gpu = world.borrow::<UniqueView<AbstractGpu>>().unwrap();
    let gpu = gpu.downcast_ref::<Gpu>().unwrap();

    // TODO(Angel): Try to make this reasonable.
    let texture = s_state
        .sub_scenes
        .get("WorkbenchScene")
        .unwrap()
        .target_texture
        .downcast_ref::<WGPUTexture>()
        .unwrap();

    let texture_id = egui_renderer.renderer.register_native_texture(
        &gpu.device,
        &texture.view,
        engine::wgpu::FilterMode::Linear,
    );

    world.add_unique(GuiResources {
        workbench_texture_id: texture_id,
    });
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

    let (hierarchy, entities) =
        tree.vertical_split(left_zone, VSplitDir::Top, VFraction::Bottom(0.3));

    tree.insert_tab(viewport, "Viewport", "Viewport");
    tree.insert_tab(logs, "General logs", "GeneralLogs");

    tree.insert_tab(right_zone, "Properties", "Properties");
    tree.insert_tab(right_zone, "Landscape", "LandscapeEditor");

    tree.insert_tab(hierarchy, "Hierarchy", "EntityHierarchy");
    tree.insert_tab(entities, "Entities", "Entities");
}

/// Renders the UI based on the Panel states.
fn render_gui_system(
    egui: UniqueView<EguiContext>,
    mut panel_state: UniqueViewMut<GuiPanelState>,
    mut start_drag: UniqueViewMut<TabDragStartPosition>,
    mut shared_data: UniqueViewMut<SharedData>,
    mut gui_resources: UniqueView<GuiResources>,
) {
    engine::egui::CentralPanel::default()
        .frame(engine::egui::Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            ..Default::default()
        })
        .show(&egui.0, |ui| {
            let rect = ToolbarWidget.ui(ui).rect;

            let viewport_size = &panel_state.tree.iter().find_map(|n| match n {
                split_panel_tree::PanelNode::Container { rect, tabs, active_tab } => {
                    if tabs.iter().find(|t| t.identification == "Viewport").is_some() {
                        Some(rect.clone())
                    } else {
                        None
                    }
                },
                _ => None,
            });          

            render_dynamic_panel_widget(
                ui,
                &mut panel_state.0,
                rect.height(),
                &mut start_drag.0,
                &mut shared_data,
                |ui, tab| match tab.identification.as_str() {
                    "Viewport" => viewport(ui, gui_resources.workbench_texture_id, &viewport_size.unwrap_or(Rect::NOTHING)),
                    "GeneralLogs" => ui.label("Logs"),
                    "Properties" => ui.label("Props"),
                    "LandscapeEditor" => ui.label("Landscape editor"),
                    "EntityHierarchy" => ui.label("Hierarchy"),
                    "Entities" => ui.label("Entities list"),

                    _ => ui.label("Error unkown zone"),
                },
            );
        });
}

fn viewport(ui: &mut Ui, texture_id: TextureId, size: &Rect) -> Response {
    let image = Image::new((texture_id, engine::egui::Vec2::new(size.width(), size.height() - 25.0)))
        .rounding(Rounding {
            nw: 0.0,
            ne: 4.0,
            sw: 4.0,
            se: 4.0,
        });

    ui.add(image)
}