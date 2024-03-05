pub mod colors;
pub mod config;
pub mod gizmo;
pub mod icons;
pub mod sections;
pub mod split_panel_tree;
pub mod style;
pub mod widgets;
pub mod windows;

use shipyard::{
    AllStoragesViewMut, EntitiesView, Get, SparseSet, Unique, UniqueView,
    UniqueViewMut, ViewMut, World,
};
use std::ops::{Deref, DerefMut};

use engine::{
    app::App,
    egui::{Frame, Margin, Rect, TextureId},
    graphics::gpu::AbstractGpu,
    plugin::{
        graphics::egui::{EguiContext, EguiRenderer},
        Pluggable,
    },
    scene::{
        components::Transform, hierarchy::Hierarchy, projection::Projection,
        scene_state::SceneState,
    },
    schedule::Schedule,
    wgpu_graphics::{buffer::WGPUTexture, gpu::Gpu},
};

use crate::gui::{
    config::GuiConfig,
    sections::viewport_section::extract_viewport_information,
    sections::{
        log_section::render_log_section,
        viewport_section::render_viewport_section,
    },
    split_panel_tree::{
        HFraction, HSplitDir, SplitPanelTree, Tab, VFraction, VSplitDir,
        ROOT_NODE,
    },
    style::{configure_fonts, configure_icon_font},
    widgets::{
        dynamic_panel_widget::{
            calculate_tag_dragging_system, render_dynamic_panel_widget,
            SharedData, TabDragStartPosition,
        },
        hierarchy_widget::{
            render_hierarchy_widget, HierarchyDeletionFlag,
            HierarchyExpandedFlag, HierarchySelectionFlag,
        },
        properties_widget::properties_widget,
    },
    widgets::{
        leading_toolbar_widget::render_leading_toolbar_widget,
        menu_toolbar_widget::render_menu_toolbar_widget,
        top_toolbar_widget::render_top_toolbar_widget,
    },
};

use self::{
    config::GuiState,
    sections::{
        asset_server_section::{
            render_asset_server, sync_egui_asset_server, EguiAssetServer,
        },
        scene_config_section::render_scene_config_section,
    },
    windows::gizmo_settings::render_gizmo_settings,
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
    landscape_texture_id: TextureId,
}

pub struct GuiPlugin;
impl Pluggable for GuiPlugin {
    fn configure(&self, app: &mut App) {
        app.world
            .add_unique(GuiPanelState(SplitPanelTree::default()));

        app.world.add_unique(TabDragStartPosition(None));
        app.world.add_unique(SharedData::default());
        app.world.add_unique(GuiState::default());
        app.world.add_unique(GuiConfig::default());
        app.world.add_unique(EguiAssetServer::default());

        app.world.run(configure_gui_system);

        app.schedule(Schedule::BeforeStart, |world| {
            register_workbench_texture(world);
        });

        app.schedule(Schedule::Update, |world| {
            sync_egui_asset_server(world);
            world.run(sync_aspect_ratio_when_viewport_changes);
        });

        app.schedule(Schedule::RequestRedraw, |world| {
            render_gui_system(world);
            world.run(calculate_tag_dragging_system);
            world.run(mantain_removed_entities_system);
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
    let mut egui_renderer =
        world.borrow::<UniqueViewMut<EguiRenderer>>().unwrap();
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

    let landscape_texture = s_state
        .sub_scenes
        .get("LandscapeScene")
        .unwrap()
        .target_texture
        .downcast_ref::<WGPUTexture>()
        .unwrap();

    let landscape_texture_id = egui_renderer.renderer.register_native_texture(
        &gpu.device,
        &landscape_texture.view,
        engine::wgpu::FilterMode::Linear,
    );

    world.add_unique(GuiResources {
        workbench_texture_id: texture_id,
        landscape_texture_id,
    });
}

/// Synchronizes the aspect ratio of the Viewport panel to prevent image stretching.
fn sync_aspect_ratio_when_viewport_changes(
    panel_state: UniqueView<GuiPanelState>,
    mut scene_state: UniqueViewMut<SceneState>,
) {
    let viewport_rect = &panel_state.find_container_rect("Viewport");
    let landscape_viewport_rect =
        &panel_state.find_container_rect("LandscapeEditor");

    scene_state
        .sub_scenes
        .iter_mut()
        .filter(|(id, scene)| {
            *id == "WorkbenchScene"
                && matches!(scene.projection, Projection::Perspective { .. })
        })
        .map(|e| e.1)
        .for_each(|s| {
            if let Projection::Perspective { aspect_ratio, .. } =
                &mut s.projection
            {
                *aspect_ratio = viewport_rect.unwrap_or(Rect::NOTHING).width()
                    / viewport_rect.unwrap_or(Rect::NOTHING).height();
            }
        });

    scene_state
        .sub_scenes
        .iter_mut()
        .filter(|(id, scene)| {
            *id == "LandscapeScene"
                && matches!(scene.projection, Projection::Perspective { .. })
        })
        .map(|e| e.1)
        .for_each(|s| {
            if let Projection::Perspective { aspect_ratio, .. } =
                &mut s.projection
            {
                *aspect_ratio = landscape_viewport_rect
                    .unwrap_or(Rect::NOTHING)
                    .width()
                    / landscape_viewport_rect.unwrap_or(Rect::NOTHING).height();
            }
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

    let (viewport, bottom_viewport) =
        tree.vertical_split(central, VSplitDir::Top, VFraction::Top(0.8));

    let (hierarchy, entities) =
        tree.vertical_split(left_zone, VSplitDir::Top, VFraction::Bottom(0.3));

    tree.insert_tab(viewport, "Viewport", "Viewport");
    tree.insert_tab(bottom_viewport, "Asset server", "AssetServer");
    tree.insert_tab(bottom_viewport, "General logs", "GeneralLogs");

    tree.insert_tab(right_zone, "Properties", "Properties");
    tree.insert_tab(right_zone, "Scenes", "ScenesConfig");

    tree.insert_tab(hierarchy, "Hierarchy", "EntityHierarchy");
    tree.insert_tab(entities, "Entities", "Entities");
}

/// Renders the UI based on the Panel states.
fn render_gui_system(world: &World) {
    // Map viewport information.
    let viewport_information = extract_viewport_information(world);

    let entities = world.borrow::<EntitiesView>().unwrap();
    let egui = world.borrow::<UniqueView<EguiContext>>().unwrap();
    let mut panel_state =
        world.borrow::<UniqueViewMut<GuiPanelState>>().unwrap();
    let mut start_drag = world
        .borrow::<UniqueViewMut<TabDragStartPosition>>()
        .unwrap();
    let mut shared_data = world.borrow::<UniqueViewMut<SharedData>>().unwrap();
    let mut entity_deletion_flags =
        world.borrow::<ViewMut<HierarchyDeletionFlag>>().unwrap();
    let mut entity_selection_flags =
        world.borrow::<ViewMut<HierarchySelectionFlag>>().unwrap();
    let mut entity_expanded_flags =
        world.borrow::<ViewMut<HierarchyExpandedFlag>>().unwrap();
    let mut hierarchy = world.borrow::<ViewMut<Hierarchy>>().unwrap();

    let mut transforms = world.borrow::<ViewMut<Transform>>().unwrap();

    // Render menu toolbar.
    render_menu_toolbar_widget(&egui.0, world);
    // Render the top toolbar.
    render_top_toolbar_widget(&egui.0);
    // Render the leading toolbar.
    render_leading_toolbar_widget(&egui.0, world);
    // Render gizmo settings if needed.
    render_gizmo_settings(&egui.0, world);

    engine::egui::CentralPanel::default()
        .frame(Frame {
            inner_margin: Margin::same(5.0),
            outer_margin: Margin::ZERO,
            ..Default::default()
        })
        .show(&egui.0, |ui| {
            render_dynamic_panel_widget(
                ui,
                &mut panel_state.0,
                &mut start_drag.0,
                &mut shared_data,
                &mut |ui, tab: &Tab| match tab.identification.as_str() {
                    "Viewport" => render_viewport_section(
                        ui,
                        &viewport_information,
                        &mut transforms,
                    ),
                    "GeneralLogs" => render_log_section(ui),
                    "Properties" => properties_widget(
                        ui,
                        &entities,
                        &mut hierarchy,
                        &mut entity_selection_flags,
                        &mut transforms,
                    ),
                    "ScenesConfig" => render_scene_config_section(ui, world),
                    "EntityHierarchy" => render_hierarchy_widget(
                        ui,
                        &entities,
                        &hierarchy,
                        &mut entity_deletion_flags,
                        &mut entity_selection_flags,
                        &mut entity_expanded_flags,
                    ),
                    "Entities" => ui.label("Entities list"),
                    "AssetServer" => render_asset_server(ui, world),

                    _ => ui.label("Error unkown zone"),
                },
            );
        });
}

fn mantain_removed_entities_system(mut all_storages: AllStoragesViewMut) {
    all_storages.delete_any::<SparseSet<HierarchyDeletionFlag>>();
}
