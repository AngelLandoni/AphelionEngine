use shipyard::UniqueView;

use crate::{graphics::components::UniqueRenderer, plugin::iced::UniqueIced};

/// A system used to execute the gui pass.
pub(crate) fn debug_gui_pass_system(u_gpu: UniqueView<UniqueRenderer>,
                                    u_iced: UniqueView<UniqueIced>) {
    println!("Running internal pass");
    u_iced.inner.lock().unwrap().render(&u_gpu.gpu);
}