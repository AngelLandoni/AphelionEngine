use crate::scene::keyboard::KeyCode;

/// A generic global event.
pub enum Event {
    Window(WindowEvent),
    Keyboard(KeyboardEvent),
    CursorMotion(f64, f64),
    UnknownOrNotImplemented,
}

/// Represents an abstraction containing all the events that can occur within
/// the window context. This functions as a lingua franca across different
/// window management systems.
pub enum WindowEvent {
    CursorMoved(f64, f64),
    CloseRequested,
    RequestRedraw,
    Resized(u32, u32),
    UnknownOrNotImplemented,
}

pub enum KeyboardEvent {
    Pressed(KeyCode),
    Released(KeyCode),
}
