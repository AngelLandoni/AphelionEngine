use ahash::AHashMap;
use shipyard::Unique;

use crate::graphics::scene::Scene;

#[derive(Unique)]
pub struct SceneState {
    pub main: Scene,
    pub sub_scenes: AHashMap<String, Scene>,
}
