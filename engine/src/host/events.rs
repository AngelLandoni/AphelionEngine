/// A generic global event.
pub enum Event {
    Window(WindowEvent),
    UnknownOrNotImplemented,
}

/// Represents an abstraction containing all the events that can occur within 
/// the window context. This functions as a lingua franca across different 
/// window management systems.
pub enum WindowEvent {
    CloseRequested,
    RequestRedraw,
    UnknownOrNotImplemented,
}