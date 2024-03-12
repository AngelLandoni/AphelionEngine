use crate::types::Size;

use super::model::Model;

#[derive(Default)]
pub struct AssetLoader {
    pub(crate) texture_to_load: Vec<(String, Vec<u8>, Size<u32>)>,
    pub(crate) models_to_load: Vec<(String, Model)>,
}

impl AssetLoader {
    /// Logs a texture to be loaded.
    pub fn load_texture(&mut self, id: String, data: Vec<u8>, size: Size<u32>) {
        self.texture_to_load.push((id, data, size));
    }

    /// Logs a model to be loaded.
    pub fn load_model(&mut self, id: String, model: Model) {
        self.models_to_load.push((id, model));
    }
}