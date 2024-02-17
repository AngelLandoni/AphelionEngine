use engine::{
    app::App,
    graphics::components::MeshComponent,
    nalgebra::{Unit, UnitQuaternion, Vector3},
    plugin::{core::clock::Clock, scene::primitives_plugin::CUBE_MESH_RESOURCE_ID, Pluggable},
    scene::{components::Transform, scene::SceneTarget},
};
use shipyard::{IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut};

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
        app.world.add_unique(LandscapeCubeRotation { angle: 0.0, count: 0.0 });

        let axis = Unit::new_normalize(Vector3::new(1.0, 2.0, 3.0));
        let rot = UnitQuaternion::from_axis_angle(&axis, 0.0);

        app.world.add_entity((
            MeshComponent(CUBE_MESH_RESOURCE_ID),
            Transform {
                position: Vector3::new(0.0, 0.0, -10.0),
                rotation: rot,
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
            SceneTarget::SubScene("LandscapeScene".to_string()),
        ));

        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    app.world.add_entity((
                        MeshComponent(CUBE_MESH_RESOURCE_ID),
                        Transform {
                            position: Vector3::new(
                                i as f32 * 5.0 + (j as f32).cos(),
                                k as f32 * 5.0 + (i as f32).sin() + (j as f32).sin(),
                                j as f32 * 5.0 + (i as f32).sin(),
                            ),
                            rotation: UnitQuaternion::default(),
                            scale: Vector3::new(1.0, 1.0, 1.0),
                        },
                        SceneTarget::SubScene("WorkbenchScene".to_string()),
                    ));
                }
            }
        }


        app.schedule(engine::schedule::Schedule::Update, |world| {
            world.run(rotate_landscape_cube);
            //world.run(sin_move);
        })
    }
}

fn rotate_landscape_cube(
    mut angle: UniqueViewMut<LandscapeCubeRotation>,
    mut transforms: ViewMut<Transform>,
    target: View<SceneTarget>,
    clock: UniqueView<Clock>,
) {
    for (t, _) in (&mut transforms, &target)
        .iter()
        .filter(|(_, t)| matches!(t, SceneTarget::SubScene(id) if id == "LandscapeScene"))
    {
        t.rotation = angle_to_quaternion(angle.angle, Vector3::y());
    }

    angle.angle += 1.0 * clock.delta_seconds() as f32;
}

fn sin_move(
    mut angle: UniqueViewMut<LandscapeCubeRotation>,
    mut transforms: ViewMut<Transform>,
    target: View<SceneTarget>,
    clock: UniqueView<Clock>,
) {
    for (t, _) in (&mut transforms, &target)
        .iter()
        .filter(|(_, t)| matches!(t, SceneTarget::SubScene(id) if id != "LandscapeScene"))
    {
        t.position = Vector3::new(t.position.x, (t.position.y + 1.0).sin() * clock.delta_seconds() as f32, t.position.z);
    }
}

fn angle_to_quaternion(angle: f32, axis: Vector3<f32>) -> UnitQuaternion<f32> {
    let axis = Unit::new_normalize(axis);
    UnitQuaternion::from_axis_angle(&axis, angle)
}