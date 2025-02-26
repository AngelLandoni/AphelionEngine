use shipyard::Unique;

use downcast_rs::{impl_downcast, Downcast};

use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

use crate::types::Size;

pub trait WindowInfoAccessible: Downcast {
    fn inner_size(&self) -> Size<u32>;
    fn scale_factor(&self) -> f64;
}
impl_downcast!(WindowInfoAccessible);

/// Represents the main application window. The current engine version does not
/// support multiple windows, and there are no immediate plans to do so. However,
/// in future projects, this abstraction may evolve to accommodate the need for
/// multiple windows.
#[derive(Unique)]
pub struct Window {
    pub(crate) accesor: Box<dyn WindowInfoAccessible>,
    pub(crate) size: Size<u32>,
    window_handle: RawWindowHandle,
    display_handle: RawDisplayHandle,
}

impl Window {
    /// Creates a new instance of Window.
    pub(crate) fn new<A: WindowInfoAccessible>(
        accesor: Box<A>,
        size: Size<u32>,
        window_handle: RawWindowHandle,
        display_handle: RawDisplayHandle,
    ) -> Self {
        Window {
            accesor,
            size,
            window_handle,
            display_handle,
        }
    }

    pub(crate) fn inner_size(&self) -> Size<u32> {
        self.accesor.inner_size()
    }
}

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
