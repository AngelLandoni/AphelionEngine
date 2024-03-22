use engine::{
    egui::{
        Response, TopBottomPanel,
    },
    graphics::components::MeshComponent,
    nalgebra::{UnitQuaternion, Vector3},
    plugin::{
        graphics::egui::EguiContext,
        scene::primitives_plugin::{
            cone_mesh_component, cube_mesh_component, cylinder_mesh_component,
            plane_mesh_component, sphere_mesh_component,
        },
    },
    scene::{components::Transform, hierarchy::Hierarchy, scene::SceneTarget},
};
use shipyard::{AddComponent, EntitiesViewMut, UniqueView, ViewMut, World};

use crate::gui::icons::{
    ADD_ENTITY, CUBE,
    MESH_CONE, MESH_CUBE, MESH_CYLINDER, MESH_PLANE, MESH_UVSPHERE,
};

use super::{
    hierarchy_widget::HierarchySelectionFlag,
    icon_button::{render_icon_context_button},
};

pub fn render_top_toolbar_widget(world: &World) -> Response {
    let egui = world.borrow::<UniqueView<EguiContext>>().unwrap();
    let mut entities = world.borrow::<EntitiesViewMut>().unwrap();
    let mut selection =
        world.borrow::<ViewMut<HierarchySelectionFlag>>().unwrap();

    let (mut meshes, mut transforms, mut scene_targets, mut hierarchies) =
        world
            .borrow::<(
                ViewMut<MeshComponent>,
                ViewMut<Transform>,
                ViewMut<SceneTarget>,
                ViewMut<Hierarchy>,
            )>()
            .unwrap();

    TopBottomPanel::top("top_toolbar")
        .resizable(false)
        .show(&egui.0, |ui| {
            //render_icon_button(ui, &POINTER, 30.0);
            render_icon_context_button(ui, &ADD_ENTITY, 30.0, |ui| {
                if ui.button("Empty").clicked() {
                    println!("Add empty entity");
                }

                ui.separator();

                let mut mesh: Option<MeshComponent> = None;
                let mut hierarchy: Option<Hierarchy> = None;

                ui.menu_button(format!("{} Shapes", CUBE), |ui| {
                    if ui.button(format!("{} Cube", MESH_CUBE)).clicked() {
                        mesh = Some(cube_mesh_component());
                        hierarchy = Some(Hierarchy::new(
                            crate::gui::icons::MESH_CUBE,
                            "Cube".to_owned(),
                        ));
                        ui.close_menu();
                    }

                    if ui.button(format!("{} Sphere", MESH_UVSPHERE)).clicked()
                    {
                        mesh = Some(sphere_mesh_component());
                        hierarchy = Some(Hierarchy::new(
                            crate::gui::icons::MESH_UVSPHERE,
                            "Sphere".to_owned(),
                        ));
                        ui.close_menu();
                    }

                    if ui
                        .button(format!("{} Cylinder", MESH_CYLINDER))
                        .clicked()
                    {
                        mesh = Some(cylinder_mesh_component());
                        hierarchy = Some(Hierarchy::new(
                            crate::gui::icons::MESH_CYLINDER,
                            "Cylinder".to_owned(),
                        ));
                        ui.close_menu()
                    }

                    if ui.button(format!("{} Cone", MESH_CONE)).clicked() {
                        mesh = Some(cone_mesh_component());
                        hierarchy = Some(Hierarchy::new(
                            crate::gui::icons::MESH_CONE,
                            "Cone".to_owned(),
                        ));
                        ui.close_menu();
                    }

                    if ui.button(format!("{} Plane", MESH_PLANE)).clicked() {
                        mesh = Some(plane_mesh_component());
                        hierarchy = Some(Hierarchy::new(
                            crate::gui::icons::MESH_PLANE,
                            "Plane".to_owned(),
                        ));
                        ui.close_menu()
                    }
                });

                if let (Some(mesh), Some(hierarchy)) = (mesh, hierarchy) {
                    let id = entities.add_entity(
                        (
                            &mut meshes,
                            &mut transforms,
                            &mut scene_targets,
                            &mut hierarchies,
                        ),
                        (
                            mesh,
                            Transform {
                                position: Vector3::new(0.0, 0.0, 0.0),
                                rotation: UnitQuaternion::default(),
                                scale: Vector3::new(1.0, 1.0, 1.0),
                            },
                            SceneTarget::SubScene("WorkbenchScene".to_string()),
                            hierarchy,
                        ),
                    );

                    selection.clear();
                    selection
                        .add_component_unchecked(id, HierarchySelectionFlag);
                }
            });
        })
        .response
}
