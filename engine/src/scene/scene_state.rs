use ahash::AHashMap;
use shipyard::Unique;

use crate::graphics::scene::Scene;

pub(crate) struct ScenesStateViewMut<'a> {
    pub(crate) main: &'a mut Scene,
    pub(crate) sub_scenes: &'a mut AHashMap<String, Scene>,
}

#[derive(Unique)]
pub struct SceneState {
    pub main: Scene,
    pub sub_scenes: AHashMap<String, Scene>,
}

impl SceneState {
    /// A handy function used to return a mutable view of each internal
    /// field.
    pub(crate) fn as_view_mut(&mut self) -> ScenesStateViewMut {
        ScenesStateViewMut {
            main: &mut self.main,
            sub_scenes: &mut self.sub_scenes,
        }
    }
}
