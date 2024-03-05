use shipyard::Component;

use crate::{
    scene::{camera::Camera, projection::Projection},
    types::Size,
};

#[derive(Component)]
pub enum SceneTarget {
    Main,
    SubScene(String),
}

#[derive(Clone)]
pub struct SceneDescriptor {
    /// Contains a debug tag.
    pub label: String,
    /// Conatins the id used to identify the `Scene`.
    pub id: String,
    /// Contains the `Camera` used in the scene.
    pub camera: Camera,
    /// Contains the `Projection` used.
    pub projection: Projection,
    /// Contains the resolution that the target texture will use.
    pub resolution: Option<Size<u32>>,
    /// Contains an state determining if the debug grid must be rendered or not.
    pub should_render_grid: bool,
    /// Determins if the scene should render a sky or not.
    pub should_render_sky: bool,
}

impl SceneDescriptor {
    /// Creates and initializes a new Scene instance, which is essential for
    /// initializing the main scene.
    pub fn main() -> Self {
        SceneDescriptor {
            label: "Main scene".to_owned(),
            id: "MainScene".to_owned(),
            camera: Camera::default(),
            projection: Projection::default(),
            resolution: Some(Size::new(2048, 1600)),
            should_render_grid: false,
            should_render_sky: false,
        }
    }
}
