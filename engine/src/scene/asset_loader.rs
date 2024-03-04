use crate::types::Size;

pub struct AssetLoader {
    pub(crate) texture_to_load: Vec<(String, Vec<u8>, Size<u32>)>,
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self {
            texture_to_load: Vec::new(),
        }
    }
}

impl AssetLoader {
    pub fn load_texture(&mut self, id: String, data: Vec<u8>, size: Size<u32>) {
        self.texture_to_load.push((id, data, size));
    }
}
