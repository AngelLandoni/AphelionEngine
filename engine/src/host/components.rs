use shipyard::Unique;

use crate::host::window::Window;

#[derive(Unique)]
pub struct UniqueWindow {
    pub(crate) host_window: Window
}

