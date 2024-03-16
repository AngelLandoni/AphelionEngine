use engine::{
    app::App,
    graphics::material::{DefaultMaterial, MaterialComponent},
    nalgebra::{Unit, UnitQuaternion, Vector3},
    plugin::{scene::primitives_plugin::cube_mesh_component, Pluggable},
    scene::{components::Transform, hierarchy::Hierarchy, scene::SceneTarget},
};

use crate::camera::EditorCamera;

pub struct WorkbenchScenePlugin;
impl Pluggable for WorkbenchScenePlugin {
    fn configure(&self, app: &mut App) {
        app.world.add_unique(EditorCamera::default());

        let axis = Unit::new_normalize(Vector3::new(1.0, 2.0, 3.0));
        let rot = UnitQuaternion::from_axis_angle(&axis, 0.0);

        app.world.add_entity((
            cube_mesh_component(),
            MaterialComponent::new(DefaultMaterial::id()),
            Transform {
                position: Vector3::new(0.0, 0.0, -10.0),
                rotation: rot,
                scale: Vector3::new(2.0, 2.0, 2.0),
            },
            SceneTarget::SubScene("WorkbenchScene".to_string()),
            Hierarchy::new(
                crate::gui::icons::MESH_CUBE,
                "Root cube".to_owned(),
            ),
        ));
    }
}
