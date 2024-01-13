use raw_window_handle::{
    HasRawWindowHandle,
    HasRawDisplayHandle,
    RawWindowHandle,
    RawDisplayHandle,
};

use crate::types::Size;

pub trait WindowInfoAccessible {
    fn inner_size(&self) -> Size<u32>;
    fn scale_factor(&self) -> f64;
}

/// Represents the main application window. The current engine version does not
/// support multiple windows, and there are no immediate plans to do so. However,
/// in future projects, this abstraction may evolve to accommodate the need for
/// multiple windows.
pub struct Window {
    pub(crate) accesor: Box<dyn WindowInfoAccessible>,
    window_handle: RawWindowHandle,
    display_handle: RawDisplayHandle,
}

impl Window {
    /// Creates a new instance of Window.
    pub(crate) fn new(accesor: Box<dyn WindowInfoAccessible>,
                      window_handle: RawWindowHandle,
                      display_handle: RawDisplayHandle) -> Self {
        Window {
            accesor,
            window_handle,
            display_handle,
        }
    }

    pub(crate) fn inner_size(&self) -> Size<u32> {
        self.accesor.inner_size()
    }

    pub(crate) fn scale_factor(&self) -> f64 {
        self.accesor.scale_factor()
    }
}

/// TODO(Angel): Ensure thread safety for everything inside this block.
/// The `Window` instance will only be accessed from one thread, but Shipyard
/// does not have this knowledge. Therefore, we need to make it `Send` + `Sync`.
/// Address this concern to ensure correct functionality.
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window_handle
    }
}
    
unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.display_handle
    }
}
    