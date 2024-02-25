pub mod colors;
pub mod gizmo;
pub mod icons;
pub mod split_panel_tree;
pub mod style;
pub mod widgets;

use egui_gizmo::{Gizmo, GizmoMode};
use shipyard::{
    AllStoragesView, AllStoragesViewMut, EntitiesView, IntoIter, SparseSet,
    Unique, UniqueView, UniqueViewMut, View, ViewMut, World,
};
use std::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};

use engine::{
    app::App,
    egui::{Image, Margin, Rect, Response, Rounding, TextureId, Ui, Widget},
    graphics::{gpu::AbstractGpu, scene::Scene},
    nalgebra::Vector3,
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
    split_panel_tree::{
        HFraction, HSplitDir, SplitPanelTree, VFraction, ROOT_NODE,
    },
    widgets::toolbar_widget::ToolbarWidget,
};

use self::{
    split_panel_tree::{Tab, VSplitDir},
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

        app.world.run(configure_gui_system);

        app.schedule(Schedule::BeforeStart, |world| {
            register_workbench_texture(world);
        });

        app.schedule(Schedule::Update, |world| {
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
                *aspect_ratio = viewport_rect.unwrap().width()
                    / viewport_rect.unwrap().height();
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
                *aspect_ratio = landscape_viewport_rect.unwrap().width()
                    / landscape_viewport_rect.unwrap().height();
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
fn render_gui_system(world: &World) {
    let entities = world.borrow::<EntitiesView>().unwrap();
    let all_storages = world.borrow::<AllStoragesView>().unwrap();
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
    let gui_resources = world.borrow::<UniqueView<GuiResources>>().unwrap();

    let mut transforms = world.borrow::<ViewMut<Transform>>().unwrap();
    let scene = world.borrow::<UniqueView<SceneState>>().unwrap();
    // TODO(Angel): Try to not hardcode this please.
    let scene = scene.sub_scenes.get("WorkbenchScene").unwrap();

    engine::egui::CentralPanel::default()
        .frame(engine::egui::Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            ..Default::default()
        })
        .show(&egui.0, |ui| {
            let rect = ToolbarWidget.ui(ui).rect;

            let viewport_rect = &panel_state.find_container_rect("Viewport");
            let landscape_viewport_rect =
                &panel_state.find_container_rect("LandscapeEditor");

            render_dynamic_panel_widget(
                ui,
                &mut panel_state.0,
                rect.height(),
                &mut start_drag.0,
                &mut shared_data,
                &mut |ui, tab: &Tab| match tab.identification.as_str() {
                    "Viewport" => viewport(
                        ui,
                        gui_resources.workbench_texture_id,
                        &viewport_rect.unwrap_or(Rect::NOTHING),
                        &scene,
                        &mut transforms,
                        &entity_selection_flags,
                    ),
                    "GeneralLogs" => ui.label("Logs"),
                    "Properties" => properties_widget(
                        ui,
                        &entities,
                        &mut hierarchy,
                        &mut entity_selection_flags,
                        &mut transforms,
                    ),
                    "LandscapeEditor" => viewport(
                        ui,
                        gui_resources.landscape_texture_id,
                        &landscape_viewport_rect.unwrap_or(Rect::NOTHING),
                        scene,
                        &mut transforms,
                        &entity_selection_flags,
                    ),
                    "EntityHierarchy" => render_hierarchy_widget(
                        ui,
                        &entities,
                        &hierarchy,
                        &mut entity_deletion_flags,
                        &mut entity_selection_flags,
                        &mut entity_expanded_flags,
                    ),
                    "Entities" => ui.label("Entities list"),

                    _ => ui.label("Error unkown zone"),
                },
            );
        });
}

fn viewport(
    ui: &mut Ui,
    texture_id: TextureId,
    size: &Rect,
    scene: &Scene,
    transforms: &mut ViewMut<Transform>,
    selections: &ViewMut<HierarchySelectionFlag>,
) -> Response {
    let image = Image::new((
        texture_id,
        engine::egui::Vec2::new(size.width(), size.height() - 25.0),
    ))
    .rounding(Rounding {
        nw: 0.0,
        ne: 4.0,
        sw: 4.0,
        se: 4.0,
    });

    let response = ui.add(image);

    let view_matrix = scene.camera.view_matrix();
    let view_matrix = egui_gizmo::mint::ColumnMatrix4 {
        x: egui_gizmo::mint::Vector4 {
            x: view_matrix.column(0).x,
            y: view_matrix.column(0).y,
            z: view_matrix.column(0).z,
            w: view_matrix.column(0).w,
        },
        y: egui_gizmo::mint::Vector4 {
            x: view_matrix.column(1).x,
            y: view_matrix.column(1).y,
            z: view_matrix.column(1).z,
            w: view_matrix.column(1).w,
        },
        z: egui_gizmo::mint::Vector4 {
            x: view_matrix.column(2).x,
            y: view_matrix.column(2).y,
            z: view_matrix.column(2).z,
            w: view_matrix.column(2).w,
        },
        w: egui_gizmo::mint::Vector4 {
            x: view_matrix.column(3).x,
            y: view_matrix.column(3).y,
            z: view_matrix.column(3).z,
            w: view_matrix.column(3).w,
        },
    };

    let proj_matrix = scene.projection.matrix();
    let proj_matrix = egui_gizmo::mint::ColumnMatrix4 {
        x: egui_gizmo::mint::Vector4 {
            x: proj_matrix.column(0).x,
            y: proj_matrix.column(0).y,
            z: proj_matrix.column(0).z,
            w: proj_matrix.column(0).w,
        },
        y: egui_gizmo::mint::Vector4 {
            x: proj_matrix.column(1).x,
            y: proj_matrix.column(1).y,
            z: proj_matrix.column(1).z,
            w: proj_matrix.column(1).w,
        },
        z: egui_gizmo::mint::Vector4 {
            x: proj_matrix.column(2).x,
            y: proj_matrix.column(2).y,
            z: proj_matrix.column(2).z,
            w: proj_matrix.column(2).w,
        },
        w: egui_gizmo::mint::Vector4 {
            x: proj_matrix.column(3).x,
            y: proj_matrix.column(3).y,
            z: proj_matrix.column(3).z,
            w: proj_matrix.column(3).w,
        },
    };

    for (t, s) in (transforms, selections).iter() {
        let model_matrix = t.as_matrix();
        let model_matrix = egui_gizmo::mint::ColumnMatrix4 {
            x: egui_gizmo::mint::Vector4 {
                x: model_matrix.column(0).x,
                y: model_matrix.column(0).y,
                z: model_matrix.column(0).z,
                w: model_matrix.column(0).w,
            },
            y: egui_gizmo::mint::Vector4 {
                x: model_matrix.column(1).x,
                y: model_matrix.column(1).y,
                z: model_matrix.column(1).z,
                w: model_matrix.column(1).w,
            },
            z: egui_gizmo::mint::Vector4 {
                x: model_matrix.column(2).x,
                y: model_matrix.column(2).y,
                z: model_matrix.column(2).z,
                w: model_matrix.column(2).w,
            },
            w: egui_gizmo::mint::Vector4 {
                x: model_matrix.column(3).x,
                y: model_matrix.column(3).y,
                z: model_matrix.column(3).z,
                w: model_matrix.column(3).w,
            },
        };

        let gizmo = Gizmo::new("My gizmo")
            .view_matrix(view_matrix)
            .projection_matrix(proj_matrix)
            .model_matrix(model_matrix)
            .mode(GizmoMode::Translate);

        if let Some(response) = gizmo.interact(ui) {
            t.position = Vector3::new(
                response.translation.x,
                response.translation.y,
                response.translation.z,
            );
            t.scale = Vector3::new(
                response.scale.x,
                response.scale.y,
                response.scale.z,
            );
        }
    }

    response
}

fn mantain_removed_entities_system(mut all_storages: AllStoragesViewMut) {
    all_storages.delete_any::<SparseSet<HierarchyDeletionFlag>>();
}
