use crate::{
    plugin::Pluggable,
    scene::{
        camera::Camera,
        keyboard::Keyboard,
        perspective::Perspective
    },
    app::App, 
};

pub struct ScenePlugin;

impl Pluggable for ScenePlugin {
    fn configure(&self, app: &mut App) {
        // Setup camera component.
        app.world.add_unique(Camera::default());
        // Setup the perspective componente, this component will be used in
        // conjuntion with the camera.
        app.world.add_unique(Perspective::default());
        // Setups the Keyboard.
        app.world.add_unique(Keyboard::default());
    }
}