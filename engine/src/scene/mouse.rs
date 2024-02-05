use shipyard::Unique;

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
