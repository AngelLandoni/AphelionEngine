use shipyard::Unique;

#[derive(Unique)]
pub struct Cursor {
    pub x: f64,
    pub y: f64,
}

impl Default for Cursor {
    fn default() -> Self {
        Self { x: Default::default(), y: Default::default() }
    }
}

#[derive(Unique)]
pub struct CursorDelta {
    pub x: f64,
    pub y: f64,
}

impl Default for CursorDelta {
    fn default() -> Self {
        Self { x: Default::default(), y: Default::default() }
    }
}