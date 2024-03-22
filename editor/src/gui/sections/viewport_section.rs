use egui_gizmo::{mint::ColumnMatrix4, Gizmo, GizmoMode, GizmoVisuals};
use engine::{
    egui::{Image, Rect, Response, Rounding, TextureId, Ui},
    nalgebra::{
        convert_unchecked, Matrix4, Quaternion, Unit, UnitQuaternion, Vector3,
        Vector4,
    },
    scene::{
        components::Transform,
        hierarchy::{get_global_transform_matrix_of_entity, Hierarchy},
        scene_state::SceneState,
    },
};
use shipyard::{
    EntityId, Get, IntoIter, IntoWithId, UniqueView, View, ViewMut, World,
};

use crate::gui::{
    config::{GizmoConfig, GizmoState, GuiConfig, GuiState},
    widgets::hierarchy_widget::HierarchySelectionFlag,
    GuiPanelState, GuiResources,
};

pub struct ViewportInformation {
    /// Contains the texture to be displayed on the viewport area.
    texture_id: TextureId,
    /// Contains the size covered by the viewport.
    size: Rect,
    /// Conatins all the positions for each gizmo.
    gizmos_transformations: Vec<(EntityId, ColumnMatrix4<f32>)>,
    /// Contains the camera view.
    camera_view: ColumnMatrix4<f32>,
    /// Contains the camera projection.
    camera_projection: ColumnMatrix4<f32>,
    /// Conatins the state of the gizmo.
    gizmo_state: GizmoState,
    /// Contains the config of the gizmo.
    gizmo_config: GizmoConfig,
}

/// Extracts the required information to render the viewport.
pub fn extract_viewport_information(world: &World) -> ViewportInformation {
    let gui_resources = world.borrow::<UniqueView<GuiResources>>().unwrap();
    let panel_state = world.borrow::<UniqueView<GuiPanelState>>().unwrap();
    let viewport_rect = panel_state.find_container_rect("Viewport");

    let transforms = world.borrow::<View<Transform>>().unwrap();
    let hierarchy = world.borrow::<View<Hierarchy>>().unwrap();
    let selection_flags =
        world.borrow::<View<HierarchySelectionFlag>>().unwrap();

    let scene = world.borrow::<UniqueView<SceneState>>().unwrap();
    let scene = scene.sub_scenes.get("WorkbenchScene").unwrap();

    let gui_state = world.borrow::<UniqueView<GuiState>>().unwrap();
    let gui_config = world.borrow::<UniqueView<GuiConfig>>().unwrap();

    let gizmos_transformations = (&transforms, &selection_flags)
        .iter()
        .with_id()
        .map(|(id, (t, _))| (id, t))
        .map(|(id, _)| id)
        .filter_map(|id| {
            Some((
                id,
                get_global_transform_matrix_of_entity(
                    id,
                    &hierarchy,
                    &transforms,
                )?,
            ))
        })
        .map(|(id, m)| (id, convert_nalgebra_matrix4(&m)))
        .collect::<Vec<_>>();

    ViewportInformation {
        texture_id: gui_resources.workbench_texture_id,
        size: viewport_rect.unwrap_or(Rect::NOTHING),
        gizmos_transformations,
        camera_view: convert_nalgebra_matrix4(&scene.camera.view_matrix()),
        camera_projection: convert_nalgebra_matrix4(&scene.projection.matrix()),
        gizmo_state: gui_state.gizmo,
        gizmo_config: gui_config.gizmo,
    }
}

pub fn render_viewport_section(
    ui: &mut Ui,
    world: &World,
    info: &ViewportInformation,
) -> Response {
    let mut transforms = world.borrow::<ViewMut<Transform>>().unwrap();

    let image = Image::new((
        info.texture_id,
        engine::egui::Vec2::new(info.size.width(), info.size.height() - 25.0),
    ))
    .rounding(Rounding {
        nw: 0.0,
        ne: 4.0,
        sw: 4.0,
        se: 4.0,
    });

    // Render the scene.
    let response = ui.add(image);

    let gizmo_type = match info.gizmo_state.kind {
        Some(g) => g,
        _ => return response,
    };

    for (e, m) in &info.gizmos_transformations {
        let gizmo = Gizmo::new("Editor gizmo")
            .view_matrix(info.camera_view)
            .projection_matrix(info.camera_projection)
            .orientation(info.gizmo_state.orientation)
            .visuals(GizmoVisuals {
                x_color: info.gizmo_config.x_color,
                z_color: info.gizmo_config.y_color,
                y_color: info.gizmo_config.z_color,
                s_color: info.gizmo_config.s_color,
                inactive_alpha: info.gizmo_config.inactive_alpha,
                highlight_alpha: info.gizmo_config.highlighted_alpha,
                stroke_width: info.gizmo_config.width,
                gizmo_size: info.gizmo_config.size,
                ..Default::default()
            })
            .model_matrix(*m)
            .mode(gizmo_type);

        if let Some(response) = gizmo.interact(ui) {
            let t: &mut Transform = match (&mut transforms).get(*e) {
                Ok(t) => t,
                _ => continue,
            };

            // The gizmo is receiving the global transform (the entity transform with
            // respecto to the parents), therefore we need to get the diff
            // between the global position and the position modified by the
            // gizmo.
            let global_m = convert_mint_matrix4(m);
            let modified_m = convert_mint_matrix4(&response.transform());

            match gizmo_type {
                GizmoMode::Translate => {
                    let diff = modified_m - global_m;

                    let offset = Vector3::new(
                        diff.column(3).x,
                        diff.column(3).y,
                        diff.column(3).z,
                    );

                    // Once the offset or delta is calculated, it's essential to determine the direction
                    // of the entity to compute the corrected offset. The adjustment in position must
                    // consider the orientation or direction of the entity to ensure accurate displacement.
                    let rot = t.rotation
                        / convert_mint_to_nalgebra(response.rotation);
                    let corrected_offset =
                        rotate_vector_by_quaternion(offset, &rot);

                    t.position += Vector3::new(
                        corrected_offset.x,
                        corrected_offset.y,
                        corrected_offset.z,
                    );
                }

                GizmoMode::Rotate => {
                    let rot = convert_mint_to_nalgebra(response.rotation);
                    let global_rot: Unit<Quaternion<f32>> =
                        convert_unchecked(global_m);
                    t.rotation *= rot / global_rot
                }

                GizmoMode::Scale => {
                    t.scale.x *= modified_m.m11 / global_m.m11;
                    t.scale.y *= modified_m.m22 / global_m.m22;
                    t.scale.z *= modified_m.m33 / global_m.m33;
                }
            };
        }
    }

    response
}

fn rotate_vector_by_quaternion(
    v: Vector3<f32>,
    q: &Quaternion<f32>,
) -> Vector3<f32> {
    // Convert the vector to a pure quaternion (zero real part)
    let v_quaternion = Quaternion::new(0.0, v.x, v.y, v.z);

    // Rotate the vector using quaternion multiplication: q * v * q_conjugate
    let rotated_v_quaternion = q * v_quaternion * q.conjugate();

    // Extract the vector part of the resulting quaternion

    Vector3::new(
        rotated_v_quaternion.i,
        rotated_v_quaternion.j,
        rotated_v_quaternion.k,
    )
}

fn convert_nalgebra_matrix4(
    matrix: &Matrix4<f32>,
) -> egui_gizmo::mint::ColumnMatrix4<f32> {
    egui_gizmo::mint::ColumnMatrix4 {
        x: egui_gizmo::mint::Vector4 {
            x: matrix.column(0).x,
            y: matrix.column(0).y,
            z: matrix.column(0).z,
            w: matrix.column(0).w,
        },
        y: egui_gizmo::mint::Vector4 {
            x: matrix.column(1).x,
            y: matrix.column(1).y,
            z: matrix.column(1).z,
            w: matrix.column(1).w,
        },
        z: egui_gizmo::mint::Vector4 {
            x: matrix.column(2).x,
            y: matrix.column(2).y,
            z: matrix.column(2).z,
            w: matrix.column(2).w,
        },
        w: egui_gizmo::mint::Vector4 {
            x: matrix.column(3).x,
            y: matrix.column(3).y,
            z: matrix.column(3).z,
            w: matrix.column(3).w,
        },
    }
}

fn convert_mint_matrix4(
    matrix: &egui_gizmo::mint::ColumnMatrix4<f32>,
) -> Matrix4<f32> {
    Matrix4::new(
        matrix.x.x, matrix.y.x, matrix.z.x, matrix.w.x, matrix.x.y, matrix.y.y,
        matrix.z.y, matrix.w.y, matrix.x.z, matrix.y.z, matrix.z.z, matrix.w.z,
        matrix.x.w, matrix.y.w, matrix.z.w, matrix.w.w,
    )
}

fn convert_mint_to_nalgebra(
    mint_quaternion: egui_gizmo::mint::Quaternion<f32>,
) -> UnitQuaternion<f32> {
    let vec4 = Vector4::new(
        mint_quaternion.v.x,
        mint_quaternion.v.y,
        mint_quaternion.v.z,
        mint_quaternion.s,
    );
    UnitQuaternion::from_quaternion(Quaternion { coords: vec4 })
}
