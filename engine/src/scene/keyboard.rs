use std::collections::HashSet;

use shipyard::Unique;

#[repr(u32)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum KeyCode {
    // Keyboard keys, we need the rest of them.
    A, B, C, D, E, F, G, H, I, J, K, L, M ,N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Arrow keys
    Left, Up, Right, Down,
    // Not yet implemented.
    Unknown,
}

impl KeyCode {
    pub(crate) fn from_u32(value: u32) -> Self {
        // Checks if the value is out of range.
        if value >= KeyCode::Unknown as u32 { return KeyCode::Unknown; }
        // Get the enum representation of the number and return it.
        unsafe { std::mem::transmute(value) }
    }
}

pub enum InputEvent {
    KeyDown(KeyCode),
    KeyUp(KeyCode),
}

#[derive(Unique)]
pub struct Keyboard {
    // List of pressed keys.
    keys_down: HashSet<KeyCode>
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            keys_down: HashSet::new(),
        }
    }
}

impl Keyboard {
    pub(crate) fn register_key(&mut self, key: KeyCode) {
        self.keys_down.insert(key);
    }

    pub(crate) fn remove_key(&mut self, key: &KeyCode) {
        self.keys_down.remove(&key);
    }

    pub fn is_key_down(&self, key: &KeyCode) -> bool {
        self.keys_down.contains(key)
    }
}
