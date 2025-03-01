use engine::{
    egui::{
        DragValue, Frame, InnerResponse, Margin, Response, ScrollArea,
        TextEdit, Ui,
    },
    graphics::components::MeshComponent,
    log::info,
    nalgebra::UnitQuaternion,
    scene::{
        assets::asset_server::AssetServer, components::Transform,
        hierarchy::Hierarchy,
    },
};
use shipyard::{
    AllStoragesView, EntitiesView, EntityId, Get, IntoIter, UniqueView,
    ViewMut, World,
};

use super::hierarchy_widget::HierarchySelectionFlag;

/// Renders the `Properties` section.
pub fn properties_widget(ui: &mut Ui, world: &World) -> Response {
    let asset_server = world.borrow::<UniqueView<AssetServer>>().unwrap();
    let entities = world.borrow::<EntitiesView>().unwrap();
    let mut hierarchy = world.borrow::<ViewMut<Hierarchy>>().unwrap();
    let mut selection_flag =
        world.borrow::<ViewMut<HierarchySelectionFlag>>().unwrap();
    let mut transforms = world.borrow::<ViewMut<Transform>>().unwrap();
    let mut mesh_components = world.borrow::<ViewMut<MeshComponent>>().unwrap();

    ui.vertical(|ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                entities
                    .iter()
                    .filter(|e| selection_flag.get(*e).is_ok())
                    .for_each(|e| {
                        render_section(
                            ui,
                            &e,
                            &asset_server,
                            &mut hierarchy,
                            &mut transforms,
                            &mut mesh_components,
                        );
                    });
            })
    })
    .response
}

fn render_section(
    ui: &mut Ui,
    entity: &EntityId,
    asset_server: &AssetServer,
    hierarchy: &mut ViewMut<Hierarchy>,
    transforms: &mut ViewMut<Transform>,
    mesh_components: &mut ViewMut<MeshComponent>,
) {
    Frame::none()
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            let h = match hierarchy.get(*entity) {
                Ok(h) => h,
                _ => return,
            };

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let InnerResponse { inner, response } =
                        ui.menu_button(format!("{}", h.icon), |ui| {
                            ScrollArea::vertical()
                                .auto_shrink([false; 2])
                                .max_height(250.0)
                                .id_source("inspector icons")
                                .show(ui, |ui| {
                                    ui.set_width(200.0);
                                    ui.set_height(250.0);
                                    ui.horizontal_wrapped(|ui| {
                                        for c in 0xE900..=0xEB99 {
                                            let c = char::from_u32(c).unwrap();
                                            if ui
                                                .button(String::from(c))
                                                .clicked()
                                            {
                                                ui.close_menu();
                                                return Some(c);
                                            }
                                        }

                                        None
                                    })
                                })
                        });
                    response.on_hover_cursor(
                        engine::egui::CursorIcon::PointingHand,
                    );

                    if let Some(icon) = inner.and_then(|r| r.inner.inner) {
                        h.icon = icon;
                    }

                    ui.add_space(4.0);
                    ui.add(TextEdit::singleline(&mut h.title));
                });

                render_transform_component_if_required(ui, entity, transforms);
                render_mesh_if_required(
                    ui,
                    entity,
                    asset_server,
                    mesh_components,
                );
            });
        });
}

fn render_transform_component_if_required(
    ui: &mut Ui,
    entity: &EntityId,
    transforms: &mut ViewMut<Transform>,
) {
    let transform: &mut Transform = match transforms.get(*entity) {
        Ok(t) => t,
        _ => return,
    };

    ui.vertical(|ui| {
        ui.label("Trasnform");
        ui.horizontal(|ui| {
            ui.label("Position");
            ui.add(
                DragValue::new(&mut transform.position.x)
                    .speed(0.01)
                    .prefix("x: "),
            );
            ui.add(
                DragValue::new(&mut transform.position.y)
                    .speed(0.01)
                    .prefix("y: "),
            );
            ui.add(
                DragValue::new(&mut transform.position.z)
                    .speed(0.01)
                    .prefix("z: "),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Scale");
            ui.add(
                DragValue::new(&mut transform.scale.x)
                    .speed(0.01)
                    .prefix("x: "),
            );
            ui.add(
                DragValue::new(&mut transform.scale.y)
                    .speed(0.01)
                    .prefix("y: "),
            );
            ui.add(
                DragValue::new(&mut transform.scale.z)
                    .speed(0.01)
                    .prefix("z: "),
            );
        });

        let (mut r_x, mut r_y, mut r_z) = transform.rotation.euler_angles();
        r_x = convert_euler_angle_to_degrees(r_x);
        r_y = convert_euler_angle_to_degrees(r_y);
        r_z = convert_euler_angle_to_degrees(r_z);
        ui.horizontal(|ui| {
            ui.label("Rotation");
            ui.add(
                DragValue::new(&mut r_x)
                    .speed(1.0)
                    .prefix("x: ")
                    .suffix("°"),
            );
            ui.add(
                DragValue::new(&mut r_y)
                    .speed(1.0)
                    .prefix("y: ")
                    .suffix("°"),
            );
            ui.add(
                DragValue::new(&mut r_z)
                    .speed(1.0)
                    .prefix("z: ")
                    .suffix("°"),
            );
        });
        r_x = r_x.to_radians();
        r_y = r_y.to_radians();
        r_z = r_z.to_radians();
        transform.rotation = UnitQuaternion::from_euler_angles(r_x, r_y, r_z);
    });
}

fn convert_euler_angle_to_degrees(angle: f32) -> f32 {
    let rest = angle.to_degrees() % 360.0;
    if rest < 0.0 {
        rest + 360.0
    } else {
        rest
    }
}

fn render_mesh_if_required(
    ui: &mut Ui,
    entity: &EntityId,
    asset_server: &AssetServer,
    mesh_components: &mut ViewMut<MeshComponent>,
) {
    let mesh: &mut MeshComponent = match mesh_components.get(*entity) {
        Ok(m) => m,
        _ => {
            info!("No mesh!");
            return;
        }
    };

    let InnerResponse { inner, response } = ui.menu_button("Mesh", |ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(250.0)
            .id_source("inspector icons")
            .show(ui, |ui| {
                ui.set_width(200.0);
                ui.set_height(250.0);
                ui.horizontal_wrapped(|ui| {
                    for mesh in asset_server.meshes() {
                        if ui.button(&mesh.0).clicked() {
                            ui.close_menu();
                            return Some(mesh);
                        }
                    }

                    None
                })
            })
    });

    response.on_hover_cursor(engine::egui::CursorIcon::PointingHand);

    if let Some(name) = inner.and_then(|r| r.inner.inner) {
        mesh.0 = name;
    }
}
