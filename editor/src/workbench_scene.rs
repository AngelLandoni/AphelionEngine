use engine::{
    app::App,
    graphics::components::MeshComponent,
    nalgebra::{Unit, UnitQuaternion, Vector3},
    plugin::{
        core::clock::Clock, scene::primitives_plugin::CUBE_MESH_RESOURCE_ID,
        Pluggable,
    },
    scene::{
        components::Transform,
        hierarchy::{add_child, Hierarchy},
        scene::SceneTarget,
    },
};
use shipyard::{
    EntitiesViewMut, EntityId, IntoIter, Unique, UniqueView,
    UniqueViewMut, View, ViewMut,
};

use crate::camera::EditorCamera;

#[derive(Unique)]
struct LandscapeCubeRotation {
    angle: f32,
    count: f32,
}

pub struct WorkbenchScenePlugin;

impl Pluggable for WorkbenchScenePlugin {
    fn configure(&self, app: &mut App) {
        app.world.add_unique(EditorCamera::default());
        app.world.add_unique(LandscapeCubeRotation {
            angle: 0.0,
            count: 0.0,
        });

        let axis = Unit::new_normalize(Vector3::new(1.0, 2.0, 3.0));
        let rot = UnitQuaternion::from_axis_angle(&axis, 0.0);

        let root_cube = app.world.add_entity((
            MeshComponent(CUBE_MESH_RESOURCE_ID),
            Transform {
                position: Vector3::new(0.0, 0.0, -10.0),
                rotation: rot,
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
            SceneTarget::SubScene("LandscapeScene".to_string()),
            Hierarchy::new(
                crate::gui::icons::MESH_CUBE,
                "Root cube".to_owned(),
            ),
        ));

        let mut last_ent = root_cube;
        let mut paris: Vec<(EntityId, EntityId)> = Vec::new();

        for i in 0..20 {
            let current = app.world.add_entity((
                MeshComponent(CUBE_MESH_RESOURCE_ID),
                Transform {
                    position: Vector3::new(i as f32 * 5.0, 0.0, -10.0),
                    rotation: rot,
                    scale: Vector3::new(1.0, 1.0, 1.0),
                },
                SceneTarget::SubScene("WorkbenchScene".to_string()),
                Hierarchy::new(
                    crate::gui::icons::MESH_UVSPHERE,
                    "Sub".to_owned(),
                ),
            ));

            paris.push((last_ent, current));
            last_ent = current;
        }

        {
            let mut h = app.world.borrow::<ViewMut<Hierarchy>>().unwrap();
            for (r, c) in paris {
                add_child(r, c, &mut h);
            }
        }

        let mut created_entities = Vec::new();

        for i in 0..1 {
            for j in 0..1 {
                for k in 0..1 {
                    let e = app.world.add_entity((
                        MeshComponent(CUBE_MESH_RESOURCE_ID),
                        Transform {
                            position: Vector3::new(
                                i as f32 * 5.0 + (j as f32).cos(),
                                k as f32 * 5.0
                                    + (i as f32).sin()
                                    + (j as f32).sin(),
                                j as f32 * 5.0 + (i as f32).sin(),
                            ),
                            rotation: UnitQuaternion::default(),
                            scale: Vector3::new(1.0, 1.0, 1.0),
                        },
                        SceneTarget::SubScene("WorkbenchScene".to_string()),
                        Hierarchy::new(
                            crate::gui::icons::MESH_CUBE,
                            format!("Ent({},{},{})", i, j, k),
                        ),
                    ));
                    created_entities.push(e);
                }
            }
        }

        app.schedule(engine::schedule::Schedule::Update, |world| {
            world.run(rotate_landscape_cube);
        })
    }
}

fn rotate_landscape_cube(
    mut angle: UniqueViewMut<LandscapeCubeRotation>,
    mut transforms: ViewMut<Transform>,
    target: View<SceneTarget>,
    clock: UniqueView<Clock>,
    _e: EntitiesViewMut,
) {
    for (t, _) in (&mut transforms, &target)
        .iter()
        .filter(|(_, t)| matches!(t, SceneTarget::SubScene(id) if id == "LandscapeScene"))
    {
        t.rotation = angle_to_quaternion(angle.angle, Vector3::y());
    }

    angle.angle += 1.0 * clock.delta_seconds() as f32;
}

fn angle_to_quaternion(angle: f32, axis: Vector3<f32>) -> UnitQuaternion<f32> {
    let axis = Unit::new_normalize(axis);
    UnitQuaternion::from_axis_angle(&axis, angle)
}
