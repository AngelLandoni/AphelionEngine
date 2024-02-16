use ahash::AHashMap;
use shipyard::Unique;

use crate::graphics::scene::Scene;

#[derive(Unique)]
pub(crate) struct SceneState {
    pub(crate) main: Scene,
    pub(crate) sub_scenes: AHashMap<String, Scene>,
}