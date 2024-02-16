use engine::{
    app::App,
    graphics::components::MeshComponent,
    nalgebra::{Unit, UnitQuaternion, Vector3},
    plugin::{scene::primitives_plugin::CUBE_MESH_RESOURCE_ID, Pluggable},
    scene::{components::Transform, scene::SceneTarget},
};

use crate::camera::EditorCamera;

pub struct WorkbenchScenePlugin;

impl Pluggable for WorkbenchScenePlugin {
    fn configure(&self, app: &mut App) {
        app.world.add_unique(EditorCamera::default());

        let axis = Unit::new_normalize(Vector3::new(1.0, 2.0, 3.0));
        let rot = UnitQuaternion::from_axis_angle(&axis, 1.78);

        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    app.world.add_entity((
                        MeshComponent(CUBE_MESH_RESOURCE_ID),
                        Transform {
                            position: Vector3::new(
                                10.0 + i as f32 * 5.0,
                                k as f32 * 5.0,
                                j as f32 * 5.0,
                            ),
                            rotation: rot,
                            scale: Vector3::new(1.0, 1.0, 1.0),
                        },
                        SceneTarget::SubScene("WorkbenchScene".to_string()),
                    ));
                }
            }
        }
    }
}
