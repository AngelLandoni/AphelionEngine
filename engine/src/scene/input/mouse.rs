use ahash::HashSet;
use shipyard::Unique;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum MouseKeyCode {
    Left,
    Right,
    Center,
}

#[derive(Unique, Default)]
pub struct Mouse {
    keys_down: HashSet<MouseKeyCode>,
}

impl Mouse {
    pub(crate) fn register_key(&mut self, key: MouseKeyCode) {
        self.keys_down.insert(key);
    }

    pub(crate) fn remove_key(&mut self, key: &MouseKeyCode) {
        self.keys_down.remove(key);
    }

    pub fn is_key_down(&self, key: MouseKeyCode) -> bool {
        self.keys_down.contains(&key)
    }
}

#[derive(Unique, Default)]
pub struct Cursor {
    pub x: f64,
    pub y: f64,
}

#[derive(Unique, Default)]
pub struct CursorDelta {
    pub x: f64,
    pub y: f64,
}

#[derive(Unique, Default)]
pub struct MouseWheelDelta {
    pub x: f64,
    pub y: f64,
}

#[derive(Unique, Default)]
pub struct MouseWheelStepDelta {
    pub x: f32,
    pub y: f32,
}
